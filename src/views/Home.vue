<template>
  <div class="home-page">
    <!-- 标题区 -->
    <div class="header">
      <div class="title-area">
        <el-icon :size="28" color="#67C23A"><DocumentChecked /></el-icon>
        <div>
          <h1>标书智能检查工具</h1>
          <p>提取检查标准 · 智能合规检查</p>
        </div>
      </div>
    </div>

    <!-- 两步操作区 -->
    <div class="steps-container">
      <!-- 第一步 -->
      <div class="step-card">
        <div class="step-header">
          <span class="step-badge">1</span>
          <span class="step-title">导入招标文件（提取标准）</span>
        </div>
        <div class="upload-box" @click="selectTenderFile" @dragover.prevent @drop.prevent="handleTenderDrop">
          <el-icon :size="32" color="#409EFF"><Upload /></el-icon>
          <p>拖拽或点击上传</p>
          <span>支持: .docx .doc .pdf</span>
        </div>
        <div v-if="appStore.hasTenderFile" class="file-tag success">
          <el-icon><CircleCheckFilled /></el-icon>
          <span>{{ appStore.project.tenderFile?.name }}</span>
          <el-button type="primary" link size="small" @click.stop="viewExtraction">查看提取</el-button>
        </div>
      </div>

      <!-- 箭头 -->
      <el-icon :size="24" color="#C0C4CC"><Right /></el-icon>

      <!-- 第二步 -->
      <div class="step-card">
        <div class="step-header">
          <span class="step-badge">2</span>
          <span class="step-title">导入投标文件（待检查）</span>
        </div>
        <div class="upload-box" @click="selectBidFile" @dragover.prevent @drop.prevent="handleBidDrop">
          <el-icon :size="32" color="#67C23A"><Document /></el-icon>
          <p>拖拽或点击上传</p>
          <span>支持: .docx .doc .pdf .xlsx .txt</span>
        </div>
        <div v-if="appStore.hasBidFile" class="file-tag success">
          <el-icon><CircleCheckFilled /></el-icon>
          <span>{{ appStore.project.bidFile?.name }}</span>
        </div>
      </div>
    </div>

    <!-- 检查选项 -->
    <div class="options-bar" v-if="appStore.hasBidFile">
      <el-checkbox v-model="checkOptions.general">通用检查</el-checkbox>
      <el-checkbox v-model="checkOptions.project" :disabled="!appStore.hasTenderFile">项目特定检查</el-checkbox>
      <el-checkbox v-model="checkOptions.custom">自定义规则</el-checkbox>
    </div>

    <!-- 操作按钮 -->
    <div class="action-bar">
      <el-button type="primary" size="large" :disabled="!appStore.canStartCheck" @click="startCheck">
        <el-icon><Lightning /></el-icon>开始检查
      </el-button>
      <el-button size="large" @click="appStore.resetProject()">重置</el-button>
    </div>

    <!-- 最近检查 -->
    <div class="recent-section" v-if="recentProjects.length > 0">
      <div class="recent-header">
        <span>最近检查</span>
      </div>
      <div class="recent-list">
        <div v-for="project in recentProjects.slice(0, 3)" :key="project.id" class="recent-item">
          <el-icon :size="14"><Document /></el-icon>
          <span class="recent-name">{{ project.name }}</span>
          <el-tag :type="project.status === 'passed' ? 'success' : 'warning'" size="small">
            {{ project.status === 'passed' ? '通过' : '有问题' }}
          </el-tag>
          <span class="recent-time">{{ project.time }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores'
import { ElMessage } from 'element-plus'
import { Upload, CircleCheckFilled, Document, Lightning, Right, DocumentChecked } from '@element-plus/icons-vue'
import { open } from '@tauri-apps/plugin-dialog'

const router = useRouter()
const appStore = useAppStore()

const recentProjects = ref([])

const checkOptions = ref({
  general: true,
  project: true,
  custom: false
})

async function selectTenderFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Documents', extensions: ['docx', 'doc', 'pdf'] }]
    })
    if (selected) {
      const fileName = selected.split(/[/\\]/).pop()
      await appStore.setTenderFile({ path: selected, name: fileName })
      ElMessage.success('招标文件上传成功，正在提取检查标准...')
    }
  } catch (error) {
    ElMessage.error('选择文件失败: ' + error.message)
  }
}

function handleTenderDrop(e) {
  const files = e.dataTransfer.files
  if (files.length > 0) {
    const file = files[0]
    if (['.docx', '.doc', '.pdf'].some(ext => file.name.endsWith(ext))) {
      appStore.setTenderFile({ path: file.path, name: file.name })
      ElMessage.success('招标文件上传成功')
    } else {
      ElMessage.warning('请选择 .docx .doc .pdf 格式')
    }
  }
}

async function selectBidFile() {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Documents', extensions: ['docx', 'doc', 'pdf', 'xlsx', 'txt'] }]
    })
    if (selected) {
      const fileName = selected.split(/[/\\]/).pop()
      appStore.setBidFile({ path: selected, name: fileName, size: 0 })
      ElMessage.success('投标文件上传成功')
    }
  } catch (error) {
    ElMessage.error('选择文件失败: ' + error.message)
  }
}

function handleBidDrop(e) {
  const files = e.dataTransfer.files
  if (files.length > 0) {
    const file = files[0]
    if (['.docx', '.doc', '.pdf', '.xlsx', '.txt'].some(ext => file.name.endsWith(ext))) {
      appStore.setBidFile({ path: file.path, name: file.name, size: 0 })
      ElMessage.success('投标文件上传成功')
    } else {
      ElMessage.warning('请选择 .docx .doc .pdf .xlsx .txt 格式')
    }
  }
}

function viewExtraction() {
  router.push('/tender-extract')
}

function startCheck() {
  router.push('/checking')
}

onMounted(() => {
  appStore.loadDefaultDesensitizeRules()
  appStore.loadCustomRules()
})
</script>

<style scoped>
.home-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 12px;
  overflow: hidden;
}

.header {
  margin-bottom: 12px;
}

.title-area {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-area h1 {
  font-size: 18px;
  font-weight: 700;
  margin: 0;
}

.title-area p {
  font-size: 11px;
  color: #909399;
  margin: 2px 0 0;
}

.steps-container {
  display: flex;
  align-items: center;
  gap: 12px;
}

.step-card {
  flex: 1;
  background: #fff;
  border-radius: 8px;
  padding: 12px;
  box-shadow: 0 1px 4px rgba(0,0,0,0.05);
  display: flex;
  flex-direction: column;
}

.step-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
}

.step-badge {
  width: 22px;
  height: 22px;
  background: linear-gradient(135deg, #67C23A, #36D1DC);
  color: #fff;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 600;
}

.step-title {
  font-size: 13px;
  font-weight: 600;
  color: #303133;
}

.upload-box {
  border: 2px dashed #dcdfe6;
  border-radius: 6px;
  padding: 20px;
  text-align: center;
  cursor: pointer;
  transition: all 0.2s;
}

.upload-box:hover {
  border-color: #409EFF;
  background: #f5f7fa;
}

.upload-box p {
  margin: 8px 0 4px;
  font-size: 12px;
  color: #606266;
}

.upload-box span {
  font-size: 10px;
  color: #909399;
}

.file-tag {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  padding: 6px 10px;
  background: #f5f7fa;
  border-radius: 4px;
  font-size: 11px;
}

.file-tag.success {
  background: #f0f9eb;
  color: #67C23A;
}

.options-bar {
  display: flex;
  gap: 16px;
  margin-top: 12px;
  padding: 10px 16px;
  background: #fff;
  border-radius: 6px;
}

.action-bar {
  display: flex;
  justify-content: center;
  gap: 12px;
  margin-top: 12px;
}

.recent-section {
  margin-top: auto;
  padding-top: 12px;
}

.recent-header {
  font-size: 12px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 8px;
}

.recent-list {
  background: #fff;
  border-radius: 6px;
  padding: 8px 12px;
}

.recent-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 0;
  border-bottom: 1px solid #f0f0f0;
}

.recent-item:last-child {
  border-bottom: none;
}

.recent-name {
  flex: 1;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.recent-time {
  font-size: 10px;
  color: #909399;
}
</style>
