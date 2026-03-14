<template>
  <div class="home-page">
    <!-- 许可证检查提示（暂时禁用）
    <el-alert 
      v-if="!licenseStatus.valid"
      title="许可证无效或未导入"
      type="error"
      :description="licenseStatus.message"
      show-icon
      :closable="false"
      style="margin-bottom: 20px"
    >
      <template #default>
        <p>{{ licenseStatus.message }}</p>
        <el-button type="primary" size="small" @click="showLicenseDialog">
          导入许可证
        </el-button>
      </template>
    </el-alert>
    
    <el-alert 
      v-else-if="usageStats.remaining_today < 5"
      :title="`今日剩余检查次数: ${usageStats.remaining_today} 次`"
      type="warning"
      show-icon
      :closable="false"
      style="margin-bottom: 20px"
    />
    -->
    
    <!-- 第一步：导入招标文件 -->
    <el-card class="upload-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <span class="step-badge">1</span>
          <span>导入招标文件（提取检查标准）</span>
        </div>
      </template>
      
      <div 
        class="upload-area"
        :class="{ dragover: tenderDragover }"
        @click="selectTenderFile"
        @dragover.prevent="tenderDragover = true"
        @dragleave.prevent="tenderDragover = false"
        @drop.prevent="handleTenderDrop"
      >
        <el-icon class="upload-icon"><Upload /></el-icon>
        <p>拖拽招标文件到这里，或点击上传</p>
        <p class="upload-hint">支持: .docx .doc .pdf</p>
      </div>
      
      <!-- 已上传的招标文件 -->
      <div v-if="appStore.hasTenderFile" class="file-info">
        <el-icon class="success-icon"><CircleCheckFilled /></el-icon>
        <div class="file-details">
          <p class="file-name">{{ appStore.project.tenderFile?.name }}</p>
          <p class="file-extract-status" v-if="appStore.project.tenderExtraction">
            ✓ 已提取: 评分表({{ appStore.project.tenderExtraction?.scoringTable?.length || 0 }}项)、
            必填章节({{ appStore.project.tenderExtraction?.requiredSections?.length || 0 }}项)、
            资质要求({{ appStore.project.tenderExtraction?.qualificationRequirements?.length || 0 }}项)
          </p>
        </div>
        <el-button type="primary" link @click="viewExtraction">
          查看提取结果
        </el-button>
      </div>
    </el-card>
    
    <!-- 第二步：导入投标文件 -->
    <el-card class="upload-card" shadow="hover">
      <template #header>
        <div class="card-header">
          <span class="step-badge">2</span>
          <span>导入投标文件（待检查）</span>
        </div>
      </template>
      
      <div 
        class="upload-area"
        :class="{ dragover: bidDragover }"
        @click="selectBidFile"
        @dragover.prevent="bidDragover = true"
        @dragleave.prevent="bidDragover = false"
        @drop.prevent="handleBidDrop"
      >
        <el-icon class="upload-icon"><Upload /></el-icon>
        <p>拖拽投标文件到这里，或点击上传</p>
        <p class="upload-hint">支持: .docx .doc .pdf .xlsx .txt</p>
      </div>
      
      <!-- 已上传的投标文件 -->
      <div v-if="appStore.hasBidFile" class="file-info">
        <el-icon class="success-icon"><CircleCheckFilled /></el-icon>
        <div class="file-details">
          <p class="file-name">{{ appStore.project.bidFile?.name }}</p>
          <p class="file-size">{{ formatFileSize(appStore.project.bidFile?.size) }}</p>
        </div>
      </div>
    </el-card>
    
    <!-- 检查选项 -->
    <el-card class="check-options-card" shadow="hover" v-if="appStore.hasBidFile">
      <template #header>
        <span>检查选项</span>
      </template>
      
      <div class="check-options">
        <el-checkbox v-model="checkOptions.general">通用检查要点</el-checkbox>
        <el-checkbox v-model="checkOptions.project" :disabled="!appStore.hasTenderFile">
          项目特定检查（招标文件提取）
        </el-checkbox>
        <el-checkbox v-model="checkOptions.custom">自定义检查规则</el-checkbox>
      </div>
    </el-card>
    
    <!-- 开始检查按钮 -->
    <div class="action-buttons">
      <el-button 
        type="primary" 
        size="large"
        :disabled="!appStore.canStartCheck"
        @click="startCheck"
      >
        <el-icon><Lightning /></el-icon>
        开始检查
      </el-button>
      
      <el-button size="large" @click="appStore.resetProject()">
        重置
      </el-button>
    </div>
    
    <!-- 最近检查 -->
    <el-card class="recent-card" shadow="hover">
      <template #header>
        <span>最近检查</span>
      </template>
      
      <el-empty v-if="recentProjects.length === 0" description="暂无检查记录" />
      
      <div v-else class="recent-list">
        <div 
          v-for="project in recentProjects" 
          :key="project.id" 
          class="recent-item"
          @click="loadProject(project)"
        >
          <el-icon><Document /></el-icon>
          <div class="recent-info">
            <p class="recent-name">{{ project.name }}</p>
            <p class="recent-time">{{ project.time }}</p>
          </div>
          <el-tag :type="project.status === 'passed' ? 'success' : 'warning'" size="small">
            {{ project.status === 'passed' ? '通过' : '有问题' }}
          </el-tag>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores'
import { ElMessage } from 'element-plus'
import { 
  Upload, 
  CircleCheckFilled, 
  Document, 
  Lightning 
} from '@element-plus/icons-vue'
import { open } from '@tauri-apps/api/dialog'
import { invoke } from '@tauri-apps/api'

const router = useRouter()
const appStore = useAppStore()

const tenderDragover = ref(false)
const bidDragover = ref(false)
const recentProjects = ref([])

const checkOptions = ref({
  general: true,
  project: true,
  custom: false
})

// 选择招标文件
async function selectTenderFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Documents',
        extensions: ['docx', 'doc', 'pdf']
      }]
    })
    
    if (selected) {
      await handleTenderFile(selected)
    }
  } catch (error) {
    ElMessage.error('选择文件失败: ' + error.message)
  }
}

// 处理招标文件拖放
async function handleTenderDrop(e) {
  tenderDragover.value = false
  const files = e.dataTransfer.files
  if (files.length > 0) {
    await handleTenderFile(files[0].path)
  }
}

// 处理招标文件
async function handleTenderFile(filePath) {
  try {
    const fileName = filePath.split(/[/\\]/).pop()
    const fileInfo = { path: filePath, name: fileName }
    
    await appStore.setTenderFile(fileInfo)
    ElMessage.success('招标文件上传成功，正在提取检查标准...')
  } catch (error) {
    ElMessage.error('招标文件处理失败: ' + error.message)
  }
}

// 选择投标文件
async function selectBidFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Documents',
        extensions: ['docx', 'doc', 'pdf', 'xlsx', 'txt']
      }]
    })
    
    if (selected) {
      await handleBidFile(selected)
    }
  } catch (error) {
    ElMessage.error('选择文件失败: ' + error.message)
  }
}

// 处理投标文件拖放
async function handleBidDrop(e) {
  bidDragover.value = false
  const files = e.dataTransfer.files
  if (files.length > 0) {
    await handleBidFile(files[0].path)
  }
}

// 处理投标文件
async function handleBidFile(filePath) {
  try {
    const fileName = filePath.split(/[/\\]/).pop()
    const fileInfo = { 
      path: filePath, 
      name: fileName,
      size: 0 // 将在后端获取
    }
    
    appStore.setBidFile(fileInfo)
    ElMessage.success('投标文件上传成功')
  } catch (error) {
    ElMessage.error('投标文件处理失败: ' + error.message)
  }
}

// 查看提取结果
function viewExtraction() {
  router.push('/tender-extract')
}

// 开始检查
async function startCheck() {
  router.push('/checking')
}

// 格式化文件大小
function formatFileSize(bytes) {
  if (!bytes) return ''
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024
    i++
  }
  return bytes.toFixed(2) + ' ' + units[i]
}

// 加载项目
function loadProject(project) {
  // TODO: 加载历史项目
}

onMounted(() => {
  appStore.loadDefaultDesensitizeRules()
  appStore.loadCustomRules()
  loadLicenseStatus()
  loadUsageStats()
})
</script>

<style scoped>
.home-page {
  max-width: 900px;
  margin: 0 auto;
  padding: 12px;
}

.upload-card {
  margin-bottom: 16px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  font-weight: 500;
}

.step-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 50%;
  font-size: 14px;
  font-weight: bold;
}

.upload-area {
  border: 2px dashed #dcdfe6;
  border-radius: 8px;
  padding: 32px;
  text-align: center;
  cursor: pointer;
  transition: all 0.3s;
}

.upload-area:hover {
  border-color: #409eff;
  background: #f0f7ff;
}

.upload-area.dragover {
  border-color: #409eff;
  background: #e6f1fc;
}

.upload-icon {
  font-size: 40px;
  color: #c0c4cc;
  margin-bottom: 12px;
}

.upload-hint {
  color: #909399;
  font-size: 11px;
  margin-top: 8px;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 16px;
  padding: 10px;
  background: #f0f9eb;
  border-radius: 8px;
}

.success-icon {
  font-size: 32px;
  color: #67c23a;
}

.file-details {
  flex: 1;
}

.file-name {
  font-weight: 500;
  color: #303133;
}

.file-size {
  color: #909399;
  font-size: 14px;
  margin-top: 4px;
}

.file-extract-status {
  color: #67c23a;
  font-size: 14px;
  margin-top: 4px;
}

.check-options-card {
  margin-bottom: 16px;
}

.check-options {
  display: flex;
  gap: 16px;
}

.action-buttons {
  display: flex;
  gap: 12px;
  margin: 16px 0;
}

.recent-card {
  margin-top: 16px;
}

.recent-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.3s;
}

.recent-item:hover {
  background: #f5f7fa;
}

.recent-info {
  flex: 1;
}

.recent-name {
  font-weight: 500;
}

.recent-time {
  color: #909399;
  font-size: 13px;
  margin-top: 4px;
}
</style>
