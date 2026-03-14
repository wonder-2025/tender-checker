<template>
  <div class="license-dialog">
    <el-tabs v-model="activeTab">
      <!-- 查看许可证状态 -->
      <el-tab-pane label="许可证状态" name="status">
        <div v-if="!licenseStatus.license" class="no-license">
          <el-empty description="未导入许可证">
            <el-button type="primary" @click="activeTab = 'import'">
              导入许可证
            </el-button>
          </el-empty>
        </div>
        
        <div v-else class="license-info">
          <el-descriptions :column="2" border>
            <el-descriptions-item label="许可证ID">
              {{ licenseStatus.license.license_id }}
            </el-descriptions-item>
            <el-descriptions-item label="状态">
              <el-tag :type="licenseStatus.valid ? 'success' : 'danger'">
                {{ licenseStatus.valid ? '有效' : '无效' }}
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="用户名">
              {{ licenseStatus.license.user_name }}
            </el-descriptions-item>
            <el-descriptions-item label="公司">
              {{ licenseStatus.license.company }}
            </el-descriptions-item>
            <el-descriptions-item label="邮箱">
              {{ licenseStatus.license.email }}
            </el-descriptions-item>
            <el-descriptions-item label="有效期至">
              {{ formatDate(licenseStatus.license.expires_at) }}
            </el-descriptions-item>
            <el-descriptions-item label="剩余天数">
              <el-tag :type="licenseStatus.days_remaining > 30 ? 'success' : 'warning'">
                {{ licenseStatus.days_remaining }} 天
              </el-tag>
            </el-descriptions-item>
            <el-descriptions-item label="每日限制">
              {{ licenseStatus.license.max_checks_per_day }} 次
            </el-descriptions-item>
          </el-descriptions>
          
          <div class="license-actions">
            <el-button type="danger" @click="removeLicense">
              移除许可证
            </el-button>
          </div>
        </div>
      </el-tab-pane>
      
      <!-- 导入许可证 -->
      <el-tab-pane label="导入许可证" name="import">
        <div class="import-section">
          <el-alert 
            title="许可证与设备绑定，不可用于其他设备"
            type="warning"
            :closable="false"
            style="margin-bottom: 20px"
          />
          
          <div class="device-info">
            <p><strong>当前设备指纹：</strong></p>
            <el-input 
              v-model="deviceFingerprint" 
              readonly 
              style="font-family: monospace"
            >
              <template #append>
                <el-button @click="copyDeviceFingerprint">
                  复制
                </el-button>
              </template>
            </el-input>
          </div>
          
          <el-divider />
          
          <el-form label-width="100px">
            <el-form-item label="许可证密钥">
              <el-input 
                v-model="licenseKey" 
                type="textarea" 
                :rows="6"
                placeholder="粘贴许可证密钥..."
              />
            </el-form-item>
            
            <el-form-item>
              <el-button 
                type="primary" 
                @click="importLicense"
                :loading="importing"
              >
                导入许可证
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </el-tab-pane>
      
      <!-- 使用统计 -->
      <el-tab-pane label="使用统计" name="usage">
        <div class="usage-stats">
          <el-row :gutter="20">
            <el-col :span="8">
              <el-statistic title="今日已用" :value="usageStats.today_count" suffix="次" />
            </el-col>
            <el-col :span="8">
              <el-statistic title="今日剩余" :value="usageStats.remaining_today" suffix="次" />
            </el-col>
            <el-col :span="8">
              <el-statistic title="每日上限" :value="usageStats.max_per_day" suffix="次" />
            </el-col>
          </el-row>
          
          <el-divider />
          
          <el-row :gutter="20">
            <el-col :span="8">
              <el-statistic title="本小时已用" :value="usageStats.hour_count" suffix="次" />
            </el-col>
            <el-col :span="8">
              <el-statistic title="本小时剩余" :value="usageStats.remaining_hour" suffix="次" />
            </el-col>
            <el-col :span="8">
              <el-statistic title="每小时上限" :value="usageStats.max_per_hour" suffix="次" />
            </el-col>
          </el-row>
        </div>
      </el-tab-pane>
      
      <!-- 审计日志 -->
      <el-tab-pane label="操作日志" name="audit">
        <el-table :data="auditLogs" style="width: 100%" max-height="400">
          <el-table-column prop="timestamp" label="时间" width="180">
            <template #default="{ row }">
              {{ formatTimestamp(row.timestamp) }}
            </template>
          </el-table-column>
          <el-table-column prop="action" label="操作" width="120" />
          <el-table-column prop="resource" label="资源" show-overflow-tooltip />
          <el-table-column prop="result" label="结果" width="80">
            <template #default="{ row }">
              <el-tag :type="row.result === 'success' ? 'success' : 'danger'" size="small">
                {{ row.result === 'success' ? '成功' : '失败' }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'

const activeTab = ref('status')
const licenseKey = ref('')
const deviceFingerprint = ref('')
const importing = ref(false)
const licenseStatus = ref({ valid: false, license: null, message: '', days_remaining: 0 })
const usageStats = ref({ today_count: 0, hour_count: 0, max_per_day: 50, max_per_hour: 20, remaining_today: 50, remaining_hour: 20 })
const auditLogs = ref([])

async function loadDeviceFingerprint() {
  try {
    const info = await invoke('get_device_info')
    deviceFingerprint.value = await invoke('get_device_info').then(info => {
      // 生成设备指纹
      return info.hostname + '-' + info.cpu_id + '-' + info.disk_id
    })
  } catch (error) {
    console.error('获取设备信息失败:', error)
  }
}

async function loadLicenseStatus() {
  try {
    const status = await invoke('get_license_status')
    licenseStatus.value = status
  } catch (error) {
    console.error('获取许可证状态失败:', error)
  }
}

async function loadUsageStats() {
  try {
    const stats = await invoke('get_usage_stats')
    usageStats.value = stats
  } catch (error) {
    console.error('获取使用统计失败:', error)
  }
}

async function loadAuditLogs() {
  try {
    const logs = await invoke('get_audit_logs', { limit: 100 })
    auditLogs.value = logs
  } catch (error) {
    console.error('获取审计日志失败:', error)
  }
}

async function importLicense() {
  if (!licenseKey.value.trim()) {
    ElMessage.warning('请输入许可证密钥')
    return
  }
  
  importing.value = true
  
  try {
    await invoke('import_license', { licenseKey: licenseKey.value })
    ElMessage.success('许可证导入成功')
    licenseKey.value = ''
    await loadLicenseStatus()
    activeTab.value = 'status'
  } catch (error) {
    ElMessage.error('导入失败: ' + error)
  } finally {
    importing.value = false
  }
}

async function removeLicense() {
  try {
    await ElMessageBox.confirm('确定要移除许可证吗？', '警告', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    
    await invoke('remove_license')
    ElMessage.success('许可证已移除')
    await loadLicenseStatus()
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('移除失败: ' + error)
    }
  }
}

function copyDeviceFingerprint() {
  navigator.clipboard.writeText(deviceFingerprint.value)
  ElMessage.success('已复制到剪贴板')
}

function formatDate(dateStr) {
  return new Date(dateStr).toLocaleDateString('zh-CN')
}

function formatTimestamp(timestamp) {
  return new Date(timestamp * 1000).toLocaleString('zh-CN')
}

onMounted(() => {
  loadDeviceFingerprint()
  loadLicenseStatus()
  loadUsageStats()
  loadAuditLogs()
})
</script>

<style scoped>
.license-dialog {
  padding: 20px;
}

.no-license {
  text-align: center;
  padding: 40px 0;
}

.license-info {
  margin-bottom: 20px;
}

.license-actions {
  margin-top: 20px;
  text-align: center;
}

.import-section {
  max-width: 600px;
  margin: 0 auto;
}

.device-info {
  margin-bottom: 20px;
}

.usage-stats {
  text-align: center;
}
</style>
