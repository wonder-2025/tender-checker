<template>
  <div class="checking-page">
    <!-- 进度步骤 -->
    <div class="progress-steps">
      <el-steps :active="currentStep" finish-status="success" align-center>
        <el-step title="解析文件" :description="stepStatus.parse" />
        <el-step title="脱敏处理" :description="stepStatus.desensitize" />
        <el-step title="AI检查" :description="stepStatus.check" />
        <el-step title="生成报告" :description="stepStatus.report" />
      </el-steps>
    </div>
    
    <!-- 当前进度详情 -->
    <div class="current-step-detail">
      <div class="step-info">
        <el-icon class="step-icon" :class="{ rotating: isProcessing }">
          <component :is="currentStepIcon" />
        </el-icon>
        <div class="step-text">
          <h3>{{ currentStepTitle }}</h3>
          <p>{{ currentStepDescription }}</p>
        </div>
      </div>
      
      <!-- 文件信息 -->
      <div v-if="currentFile" class="file-info">
        <el-icon><Document /></el-icon>
        <span>{{ currentFile.name }}</span>
        <span class="file-size">({{ formatSize(currentFile.size) }})</span>
      </div>
    </div>
    
    <!-- 进度条 -->
    <div class="progress-bar">
      <el-progress 
        :percentage="overallProgress" 
        :status="progressStatus"
        :stroke-width="12"
      />
    </div>
    
    <!-- 检查项目列表 -->
    <div class="check-items">
      <h4>检查项目 ({{ completedItems }}/{{ totalItems }})</h4>
      <div class="items-list">
        <div 
          v-for="item in checkItems" 
          :key="item.id"
          class="check-item"
          :class="item.status"
        >
          <el-icon v-if="item.status === 'done'"><CircleCheckFilled /></el-icon>
          <el-icon v-else-if="item.status === 'error'"><CircleCloseFilled /></el-icon>
          <el-icon v-else-if="item.status === 'processing'" class="rotating"><Loading /></el-icon>
          <el-icon v-else><RemoveFilled /></el-icon>
          
          <span class="item-name">{{ item.name }}</span>
          
          <el-tag v-if="item.result" :type="item.result" size="small">
            {{ item.resultText }}
          </el-tag>
        </div>
      </div>
    </div>
    
    <!-- 实时日志 -->
    <div class="log-panel">
      <div class="log-header">
        <span>实时日志</span>
        <el-button size="small" @click="clearLogs">清空</el-button>
      </div>
      <div class="log-content" ref="logContainer">
        <div 
          v-for="(log, index) in logs" 
          :key="index"
          class="log-item"
          :class="log.level"
        >
          <span class="log-time">{{ log.time }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
    </div>
    
    <!-- 操作按钮 -->
    <div class="action-buttons">
      <el-button v-if="isProcessing" type="danger" @click="cancelCheck">
        取消检查
      </el-button>
      <el-button v-else-if="isCompleted" type="primary" @click="viewResults">
        查看结果
      </el-button>
      <el-button v-else type="primary" @click="startCheck">
        开始检查
      </el-button>
    </div>
    
    <!-- 智能提示 -->
    <el-alert 
      v-if="showTip"
      :title="currentTip"
      type="info"
      show-icon
      :closable="false"
      style="margin-top: 20px"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores'
import { ElMessage, ElMessageBox } from 'element-plus'
import { 
  Document, Loading, CircleCheckFilled, CircleCloseFilled, RemoveFilled 
} from '@element-plus/icons-vue'

const router = useRouter()
const appStore = useAppStore()

const currentStep = ref(0)
const overallProgress = ref(0)
const isProcessing = ref(false)
const isCompleted = ref(false)
const logs = ref([])
const logContainer = ref(null)
const checkItems = ref([])
const showTip = ref(true)

const stepStatus = ref({
  parse: '等待中',
  desensitize: '等待中',
  check: '等待中',
  report: '等待中'
})

const currentFile = computed(() => appStore.project.bidFile)

const stepIcons = [Document, Document, Document, Document]
const currentStepIcon = computed(() => stepIcons[currentStep.value] || Document)

const currentStepTitle = computed(() => {
  const titles = ['解析文件', '脱敏处理', 'AI检查', '生成报告']
  return titles[currentStep.value] || '准备中'
})

const currentStepDescription = computed(() => {
  return stepStatus.value[['parse', 'desensitize', 'check', 'report'][currentStep.value]] || ''
})

const progressStatus = computed(() => {
  if (isCompleted.value) return 'success'
  if (!isProcessing.value && overallProgress.value === 0) return ''
  return null
})

const completedItems = computed(() => checkItems.value.filter(i => i.status === 'done').length)
const totalItems = computed(() => checkItems.value.length)

const tips = [
  '💡 检查过程中，敏感信息会自动脱敏',
  '💡 检查完成后可以导出Word或PDF报告',
  '💡 可以在"设置"中自定义检查规则',
  '💡 历史记录会自动保存，方便后续查看'
]

const currentTip = computed(() => {
  return tips[Math.floor(Date.now() / 5000) % tips.length]
})

let tipTimer = null

function addLog(level, message) {
  const time = new Date().toLocaleTimeString()
  logs.value.push({ time, level, message })
  
  // 自动滚动到底部
  setTimeout(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  }, 50)
}

function clearLogs() {
  logs.value = []
}

function formatSize(bytes) {
  if (!bytes) return ''
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  while (bytes >= 1024 && i < units.length - 1) {
    bytes /= 1024
    i++
  }
  return bytes.toFixed(2) + ' ' + units[i]
}

async function startCheck() {
  if (!currentFile.value) {
    ElMessage.warning('请先导入投标文件')
    return
  }
  
  isProcessing.value = true
  isCompleted.value = false
  overallProgress.value = 0
  currentStep.value = 0
  
  clearLogs()
  addLog('info', '开始检查...')
  
  // 初始化检查项
  checkItems.value = [
    { id: 1, name: '项目名称一致性', status: 'pending' },
    { id: 2, name: '金额一致性', status: 'pending' },
    { id: 3, name: '友商标识', status: 'pending' },
    { id: 4, name: '错别字检查', status: 'pending' },
    { id: 5, name: '日期检查', status: 'pending' }
  ]
  
  try {
    // 步骤1: 解析文件
    await simulateStep('parse', '正在解析文件...', 20)
    
    // 步骤2: 脱敏处理
    await simulateStep('desensitize', '正在进行脱敏处理...', 40)
    
    // 步骤3: AI检查
    await simulateStep('check', '正在进行AI检查...', 70)
    await runCheckItems()
    
    // 步骤4: 生成报告
    await simulateStep('report', '正在生成报告...', 100)
    
    isCompleted.value = true
    addLog('success', '检查完成！')
    ElMessage.success('检查完成')
    
  } catch (error) {
    addLog('error', `检查失败: ${error}`)
    ElMessage.error('检查失败: ' + error)
  } finally {
    isProcessing.value = false
  }
}

async function simulateStep(stepName, message, targetProgress) {
  stepStatus.value[stepName] = '进行中'
  addLog('info', message)
  
  // 模拟进度
  while (overallProgress.value < targetProgress) {
    await new Promise(r => setTimeout(r, 100))
    overallProgress.value += 1
  }
  
  stepStatus.value[stepName] = '已完成'
  currentStep.value += 1
}

async function runCheckItems() {
  for (let i = 0; i < checkItems.value.length; i++) {
    const item = checkItems.value[i]
    item.status = 'processing'
    addLog('info', `正在检查: ${item.name}`)
    
    await new Promise(r => setTimeout(r, 800))
    
    // 模拟检查结果
    const results = ['success', 'error', 'warning']
    const result = results[Math.floor(Math.random() * 3)]
    
    item.status = 'done'
    item.result = result
    item.resultText = result === 'success' ? '通过' : result === 'error' ? '错误' : '警告'
    
    if (result === 'success') {
      addLog('success', `${item.name}: 通过`)
    } else {
      addLog('warning', `${item.name}: 发现问题`)
    }
  }
}

function cancelCheck() {
  ElMessageBox.confirm('确定要取消检查吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    isProcessing.value = false
    addLog('warning', '检查已取消')
    ElMessage.info('检查已取消')
  }).catch(() => {})
}

function viewResults() {
  router.push('/result')
}

onMounted(() => {
  // 定时切换提示
  tipTimer = setInterval(() => {
    showTip.value = false
    setTimeout(() => showTip.value = true, 100)
  }, 5000)
})

onUnmounted(() => {
  if (tipTimer) {
    clearInterval(tipTimer)
  }
})
</script>

<style scoped>
.checking-page {
  max-width: 900px;
  margin: 0 auto;
  padding: 20px;
}

.progress-steps {
  margin-bottom: 40px;
}

.current-step-detail {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  background: var(--card-bg);
  border-radius: 12px;
  margin-bottom: 24px;
  border: 1px solid var(--border-color);
}

.step-info {
  display: flex;
  align-items: center;
  gap: 16px;
}

.step-icon {
  font-size: 32px;
  color: #409EFF;
}

.step-icon.rotating {
  animation: rotate 1s linear infinite;
}

@keyframes rotate {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.step-text h3 {
  margin: 0;
  color: var(--text-primary);
}

.step-text p {
  margin: 4px 0 0;
  color: var(--text-secondary);
  font-size: 14px;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-secondary);
  font-size: 14px;
}

.file-size {
  color: var(--text-tertiary);
}

.progress-bar {
  margin-bottom: 24px;
}

.check-items {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 20px;
  margin-bottom: 24px;
  border: 1px solid var(--border-color);
}

.check-items h4 {
  margin: 0 0 16px;
  color: var(--text-primary);
}

.items-list {
  display: grid;
  gap: 8px;
}

.check-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
}

.check-item.done {
  color: var(--text-secondary);
}

.check-item.processing {
  color: #409EFF;
}

.check-item .el-icon.rotating {
  animation: rotate 1s linear infinite;
}

.item-name {
  flex: 1;
}

.log-panel {
  background: var(--card-bg);
  border-radius: 12px;
  border: 1px solid var(--border-color);
  margin-bottom: 24px;
  overflow: hidden;
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-secondary);
  font-size: 14px;
}

.log-content {
  height: 200px;
  overflow-y: auto;
  padding: 12px;
  font-family: monospace;
  font-size: 12px;
  background: var(--bg-secondary);
}

.log-item {
  display: flex;
  gap: 12px;
  padding: 4px 0;
}

.log-time {
  color: var(--text-tertiary);
  min-width: 70px;
}

.log-message {
  color: var(--text-primary);
}

.log-item.info .log-message { color: var(--text-secondary); }
.log-item.success .log-message { color: #67C23A; }
.log-item.warning .log-message { color: #E6A23C; }
.log-item.error .log-message { color: #F56C6C; }

.action-buttons {
  display: flex;
  justify-content: center;
  gap: 16px;
}

/* 响应式 */
@media (max-width: 768px) {
  .current-step-detail {
    flex-direction: column;
    gap: 16px;
  }
  
  .step-info {
    flex-direction: column;
    text-align: center;
  }
}
</style>
