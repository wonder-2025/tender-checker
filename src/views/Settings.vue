<template>
  <div class="settings-page">
    <el-tabs v-model="activeTab">
      <!-- 错误日志上报 -->
      <el-tab-pane label="错误日志" name="errorReport">
        <el-card shadow="never">
          <el-alert 
            type="info" 
            :closable="false"
            style="margin-bottom: 16px"
          >
            <template #title>
              <strong>错误日志上报</strong>
            </template>
            启用后，应用遇到错误时会自动将日志发送到服务器，帮助开发者快速定位和修复问题。
          </el-alert>
          
          <el-form label-width="120px" class="config-form">
            <el-form-item label="启用上报">
              <el-switch v-model="settingsStore.errorReport.enabled" />
              <div class="hint" style="margin-top: 4px">启用后，遇到错误时自动发送日志</div>
            </el-form-item>
            
            <el-form-item label="服务器地址" v-if="settingsStore.errorReport.enabled">
              <el-input 
                v-model="settingsStore.errorReport.serverUrl" 
                placeholder="错误日志服务器地址"
              />
              <div class="hint" style="margin-top: 4px">接收错误日志的服务器地址</div>
            </el-form-item>
            
            <el-form-item v-if="settingsStore.errorReport.enabled">
              <el-button type="primary" @click="testErrorReportConnection" :loading="testingErrorReport">
                测试连接
              </el-button>
              <span v-if="errorReportStatus" :style="{ color: errorReportStatus === '成功' ? '#67C23A' : '#F56C6C', marginLeft: '10px' }">
                {{ errorReportStatus }}
              </span>
            </el-form-item>
          </el-form>
        </el-card>
      </el-tab-pane>
      
      <!-- API配置 -->
      <el-tab-pane label="API配置" name="api">
        <el-card shadow="never">
          <el-form :model="apiForm" label-width="120px" class="config-form">
            <el-form-item label="服务商">
              <el-select v-model="apiForm.provider" @change="onProviderChange">
                <el-option label="百度千帆" value="baidu" />
                <el-option label="阿里通义" value="aliyun" />
                <el-option label="OpenAI" value="openai" />
                <el-option label="自定义" value="custom" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="API Key">
              <el-input 
                v-model="apiForm.apiKey" 
                type="password" 
                show-password
                placeholder="请输入API Key"
              />
            </el-form-item>
            
            <el-form-item label="Secret Key" v-if="apiForm.provider === 'baidu'">
              <el-input 
                v-model="apiForm.secretKey" 
                type="password" 
                show-password
                placeholder="请输入Secret Key"
              />
            </el-form-item>
            
            <el-form-item label="模型">
              <el-select v-model="apiForm.model">
                <el-option 
                  v-for="model in availableModels" 
                  :key="model.value" 
                  :label="model.label" 
                  :value="model.value"
                />
              </el-select>
            </el-form-item>
            
            <el-form-item label="自定义端点" v-if="apiForm.provider === 'custom'">
              <el-input 
                v-model="apiForm.baseUrl" 
                placeholder="https://api.example.com/v1"
              />
            </el-form-item>
            
            <el-form-item>
              <el-button type="primary" @click="testApi">测试连接</el-button>
              <el-button @click="saveApiConfig">保存配置</el-button>
            </el-form-item>
          </el-form>
        </el-card>
      </el-tab-pane>
      
      <!-- 脱敏配置 -->
      <el-tab-pane label="脱敏配置" name="desensitize">
        <el-card shadow="never">
          <div class="section-header">
            <h4>预设规则</h4>
            <span class="hint">可开关，不可删除</span>
          </div>
          
          <el-checkbox-group v-model="enabledRules" class="rules-grid">
            <el-checkbox 
              v-for="rule in defaultRules" 
              :key="rule.id" 
              :label="rule.id"
            >
              {{ rule.name }}
            </el-checkbox>
          </el-checkbox-group>
          
          <el-divider />
          
          <div class="section-header">
            <h4>自定义规则</h4>
            <el-button type="primary" size="small" @click="showAddRuleDialog">
              + 添加规则
            </el-button>
          </div>
          
          <el-table :data="customDesensitizeRules" style="width: 100%">
            <el-table-column prop="name" label="规则名称" />
            <el-table-column prop="pattern" label="匹配模式" show-overflow-tooltip />
            <el-table-column prop="replacement" label="替换为" />
            <el-table-column label="操作" width="150">
              <template #default="{ row }">
                <el-button type="primary" link @click="editRule(row)">编辑</el-button>
                <el-button type="danger" link @click="deleteRule(row)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>
      
      <!-- Debug设置 -->
      <el-tab-pane label="调试设置" name="debug">
        <el-card shadow="never">
          <el-alert 
            type="warning" 
            :closable="false"
            style="margin-bottom: 16px"
          >
            <template #title>
              <strong>Debug模式用于开发调试</strong>
            </template>
            启用后会生成详细日志，记录脱敏过程、API请求响应、文件操作等。日志文件可能包含敏感信息（已脱敏），请妥善保管。
          </el-alert>
          
          <el-form :model="debugForm" label-width="120px" class="config-form">
            <el-form-item label="启用Debug模式">
              <el-switch v-model="debugForm.enabled" />
            </el-form-item>
            
            <el-form-item label="日志级别">
              <el-select v-model="debugForm.logLevel" :disabled="!debugForm.enabled">
                <el-option label="INFO - 基本信息" value="info" />
                <el-option label="DEBUG - 详细信息" value="debug" />
                <el-option label="TRACE - 最详细" value="trace" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="记录内容">
              <el-checkbox-group v-model="debugForm.logItems" :disabled="!debugForm.enabled">
                <el-checkbox label="脱敏过程" value="desensitize" />
                <el-checkbox label="API请求" value="apiRequest" />
                <el-checkbox label="API响应" value="apiResponse" />
                <el-checkbox label="文件操作" value="fileOperation" />
                <el-checkbox label="检查过程" value="checkProcess" />
              </el-checkbox-group>
            </el-form-item>
            
            <el-form-item label="日志文件路径">
              <el-input v-model="debugForm.logFilePath" disabled>
                <template #append>
                  <el-button @click="openLogFolder">打开目录</el-button>
                </template>
              </el-input>
            </el-form-item>
            
            <el-form-item label="日志大小">
              <span>{{ debugForm.logFileSize || '0 KB' }}</span>
            </el-form-item>
            
            <el-form-item>
              <el-button @click="viewLogFile" :disabled="!debugForm.enabled">
                <el-icon><View /></el-icon>
                查看日志
              </el-button>
              <el-button @click="clearLogFile" :disabled="!debugForm.enabled">
                <el-icon><Delete /></el-icon>
                清空日志
              </el-button>
              <el-button @click="exportLogFile" :disabled="!debugForm.enabled">
                <el-icon><Download /></el-icon>
                导出日志
              </el-button>
            </el-form-item>
          </el-form>
          
          <el-divider />
          
          <div class="debug-preview">
            <h4>日志格式预览</h4>
            <pre class="log-sample">[2026-03-11 09:35:00.123] [DEBUG] [DESENSITIZE]
  Original: 张三，身份证号：123456789012345678
  Desensitized: 张三，身份证号：1234****5678
  Applied rules: [身份证号]

[2026-03-11 09:35:01.456] [DEBUG] [API_REQUEST]
  Provider: 百度千帆
  Endpoint: https://aip.baidubce.com/...
  Headers: Content-Type: application/json
  Body: {"messages":[{"role":"user","content":"..."}]}

[2026-03-11 09:35:03.789] [DEBUG] [API_RESPONSE]
  Status: 200
  Body: {"result":"检查结果..."}</pre>
          </div>
        </el-card>
      </el-tab-pane>
    </el-tabs>
    
    <!-- 添加脱敏规则对话框 -->
    <el-dialog v-model="ruleDialogVisible" title="添加脱敏规则" width="500px">
      <el-form :model="ruleForm" label-width="100px">
        <el-form-item label="规则名称">
          <el-input v-model="ruleForm.name" placeholder="如：合同编号" />
        </el-form-item>
        <el-form-item label="匹配模式">
          <el-input 
            v-model="ruleForm.pattern" 
            placeholder="正则表达式，如：HT-\d{8}"
          />
        </el-form-item>
        <el-form-item label="替换为">
          <el-input v-model="ruleForm.replacement" placeholder="如：[合同编号]" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="ruleDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveRule">保存</el-button>
      </template>
    </el-dialog>
    
    <!-- 添加检查规则对话框 -->
    <el-dialog v-model="checkRuleDialogVisible" title="添加检查规则" width="600px">
      <el-form :model="checkRuleForm" label-width="100px">
        <el-form-item label="规则名称">
          <el-input v-model="checkRuleForm.name" placeholder="如：特殊字符检查" />
        </el-form-item>
        <el-form-item label="检查说明">
          <el-input 
            v-model="checkRuleForm.description" 
            type="textarea"
            :rows="2"
            placeholder="描述这个规则要检查什么"
          />
        </el-form-item>
        <el-form-item label="提示词">
          <el-input 
            v-model="checkRuleForm.promptTemplate" 
            type="textarea"
            :rows="4"
            placeholder="发给AI的检查指令"
          />
        </el-form-item>
        <el-form-item label="严重程度">
          <el-select v-model="checkRuleForm.severity">
            <el-option label="错误" value="error" />
            <el-option label="警告" value="warning" />
            <el-option label="提示" value="info" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="checkRuleDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveCheckRule">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, onMounted, computed } from 'vue'
import { useAppStore } from '@/stores'
import { useSettingsStore } from '@/stores/settings'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api'
import { View, Delete, Download } from '@element-plus/icons-vue'

const appStore = useAppStore()
const settingsStore = useSettingsStore()
const activeTab = ref('errorReport')

// 错误日志测试状态
const testingErrorReport = ref(false)
const errorReportStatus = ref('')

// 测试错误日志服务器连接
async function testErrorReportConnection() {
  testingErrorReport.value = true
  errorReportStatus.value = ''
  
  try {
    const result = await settingsStore.testConnection()
    if (result.success) {
      errorReportStatus.value = '成功'
      ElMessage.success('连接测试成功')
    } else {
      errorReportStatus.value = '失败'
      ElMessage.error('连接失败: ' + result.message)
    }
  } catch (error) {
    errorReportStatus.value = '失败'
    ElMessage.error('连接失败: ' + error.message)
  } finally {
    testingErrorReport.value = false
  }
}

// API配置
const apiForm = reactive({
  provider: 'baidu',
  apiKey: '',
  secretKey: '',
  model: 'ERNIE-4.0',
  baseUrl: ''
})

const models = {
  baidu: [
    { label: 'ERNIE-4.0', value: 'ERNIE-4.0' },
    { label: 'ERNIE-3.5', value: 'ERNIE-3.5' }
  ],
  aliyun: [
    { label: 'Qwen-Max', value: 'qwen-max' },
    { label: 'Qwen-Plus', value: 'qwen-plus' }
  ],
  openai: [
    { label: 'GPT-4', value: 'gpt-4' },
    { label: 'GPT-3.5-Turbo', value: 'gpt-3.5-turbo' }
  ],
  custom: []
}

const availableModels = computed(() => models[apiForm.provider] || [])

function onProviderChange() {
  apiForm.model = availableModels.value[0]?.value || ''
}

async function testApi() {
  try {
    const result = await invoke('test_api_connection', { config: apiForm })
    if (result.success) {
      ElMessage.success('API连接成功')
    } else {
      ElMessage.error('API连接失败: ' + result.error)
    }
  } catch (error) {
    ElMessage.error('测试失败: ' + error.message)
  }
}

function saveApiConfig() {
  appStore.setApiConfig(apiForm)
  ElMessage.success('配置已保存')
}

// 脱敏规则
const defaultRules = ref([])
const enabledRules = ref([])
const customDesensitizeRules = ref([])
const ruleDialogVisible = ref(false)
const ruleForm = reactive({
  name: '',
  pattern: '',
  replacement: ''
})

async function loadDefaultRules() {
  try {
    const rules = await invoke('get_default_rules')
    defaultRules.value = rules
    enabledRules.value = rules.filter(r => r.enabled).map(r => r.id)
  } catch (error) {
    console.error('加载默认规则失败:', error)
  }
}

function showAddRuleDialog() {
  Object.assign(ruleForm, { name: '', pattern: '', replacement: '' })
  ruleDialogVisible.value = true
}

async function saveRule() {
  try {
    await invoke('create_desensitize_rule', { rule: ruleForm })
    ElMessage.success('规则已添加')
    ruleDialogVisible.value = false
    loadCustomRules()
  } catch (error) {
    ElMessage.error('添加失败: ' + error.message)
  }
}

// 检查规则
const showAllGeneralRules = ref(false)
const generalCheckRules = ref([])
const enabledGeneralRules = ref([])
const customCheckRules = ref([])
const checkRuleDialogVisible = ref(false)
const checkRuleForm = reactive({
  name: '',
  description: '',
  promptTemplate: '',
  severity: 'warning'
})

async function loadCustomRules() {
  try {
    const rules = await invoke('get_custom_check_rules')
    customCheckRules.value = rules
  } catch (error) {
    console.error('加载自定义规则失败:', error)
  }
}

function showAddCheckRuleDialog() {
  Object.assign(checkRuleForm, {
    name: '',
    description: '',
    promptTemplate: '',
    severity: 'warning'
  })
  checkRuleDialogVisible.value = true
}

async function saveCheckRule() {
  try {
    await invoke('create_custom_check_rule', { request: checkRuleForm })
    ElMessage.success('规则已添加')
    checkRuleDialogVisible.value = false
    loadCustomRules()
  } catch (error) {
    ElMessage.error('添加失败: ' + error.message)
  }
}

function getSeverityType(severity) {
  const types = { error: 'danger', warning: 'warning', info: 'info' }
  return types[severity] || 'info'
}

function getSeverityLabel(severity) {
  const labels = { error: '错误', warning: '警告', info: '提示' }
  return labels[severity] || severity
}

async function toggleRule(rule) {
  try {
    await invoke('toggle_custom_check_rule', { id: rule.id })
  } catch (error) {
    ElMessage.error('操作失败: ' + error.message)
  }
}

async function exportRules() {
  try {
    const json = await invoke('export_custom_check_rules')
    // TODO: 保存到文件
    ElMessage.success('规则已导出')
  } catch (error) {
    ElMessage.error('导出失败: ' + error.message)
  }
}

async function importRules() {
  // TODO: 从文件导入
}

// Debug设置
const debugForm = reactive({
  enabled: false,
  logLevel: 'debug',
  logItems: ['desensitize', 'apiRequest', 'apiResponse', 'fileOperation', 'checkProcess'],
  logFilePath: '',
  logFileSize: ''
})

async function loadDebugConfig() {
  try {
    const config = await invoke('get_debug_config')
    if (config) {
      debugForm.enabled = config.enabled
      debugForm.logLevel = config.log_level
      debugForm.logItems = config.log_items || []
      debugForm.logFilePath = config.log_file_path
      debugForm.logFileSize = config.log_file_size
    }
  } catch (error) {
    // 默认值
    debugForm.logFilePath = '未设置'
  }
}

async function saveDebugConfig() {
  try {
    await invoke('save_debug_config', { 
      config: {
        enabled: debugForm.enabled,
        log_level: debugForm.logLevel,
        log_items: debugForm.logItems
      }
    })
    ElMessage.success('Debug配置已保存')
  } catch (error) {
    ElMessage.error('保存失败: ' + error.message)
  }
}

async function openLogFolder() {
  try {
    await invoke('open_log_folder')
  } catch (error) {
    ElMessage.error('打开目录失败: ' + error.message)
  }
}

async function viewLogFile() {
  try {
    const content = await invoke('read_log_file', { lines: 100 })
    // TODO: 显示日志查看对话框
    console.log('Log content:', content)
  } catch (error) {
    ElMessage.error('读取日志失败: ' + error.message)
  }
}

async function clearLogFile() {
  try {
    await invoke('clear_log_file')
    ElMessage.success('日志已清空')
    debugForm.logFileSize = '0 KB'
  } catch (error) {
    ElMessage.error('清空失败: ' + error.message)
  }
}

async function exportLogFile() {
  try {
    const path = await invoke('export_log_file')
    ElMessage.success('日志已导出到: ' + path)
  } catch (error) {
    ElMessage.error('导出失败: ' + error.message)
  }
}

// 监听debug开关变化，自动保存
import { watch } from 'vue'
watch(() => debugForm.enabled, (val) => {
  saveDebugConfig()
})
watch(() => debugForm.logLevel, () => {
  if (debugForm.enabled) saveDebugConfig()
})
watch(() => debugForm.logItems, () => {
  if (debugForm.enabled) saveDebugConfig()
}, { deep: true })

onMounted(() => {
  loadDefaultRules()
  loadCustomRules()
  loadDebugConfig()
})
</script>

<style scoped>
.settings-page {
  max-width: 900px;
  margin: 0 auto;
}

.config-form {
  max-width: 500px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.section-header h4 {
  margin: 0;
}

.hint {
  color: #909399;
  font-size: 13px;
}

.rules-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}

.rules-list {
  max-height: 400px;
  overflow-y: auto;
  transition: max-height 0.3s;
}

.rules-list.collapsed {
  max-height: 200px;
}

.rule-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.rule-desc {
  color: #909399;
  font-size: 13px;
}

.import-export {
  margin-top: 20px;
  display: flex;
  gap: 12px;
}

.debug-preview {
  background: #f5f7fa;
  border-radius: 8px;
  padding: 16px;
}

.dark .debug-preview {
  background: #1e293b;
}

.debug-preview h4 {
  margin: 0 0 12px 0;
  color: #606266;
}

.dark .debug-preview h4 {
  color: #94a3b8;
}

.log-sample {
  background: #1e293b;
  color: #10b981;
  padding: 16px;
  border-radius: 6px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  line-height: 1.6;
  overflow-x: auto;
  margin: 0;
  white-space: pre-wrap;
}

.dark .log-sample {
  background: #0f172a;
}
</style>
