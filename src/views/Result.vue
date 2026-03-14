<template>
  <div class="result-page">
    <!-- 统计卡片 -->
    <div class="stats-cards">
      <div class="stat-card error">
        <div class="stat-number">{{ errorCount }}</div>
        <div class="stat-label">错误</div>
      </div>
      <div class="stat-card warning">
        <div class="stat-number">{{ warningCount }}</div>
        <div class="stat-label">警告</div>
      </div>
      <div class="stat-card info">
        <div class="stat-number">{{ infoCount }}</div>
        <div class="stat-label">提示</div>
      </div>
      <div class="stat-card score">
        <div class="stat-number">{{ score }}</div>
        <div class="stat-label">评分</div>
      </div>
    </div>
    
    <!-- 筛选栏 -->
    <div class="filter-bar">
      <el-radio-group v-model="filter" size="small">
        <el-radio-button label="all">全部</el-radio-button>
        <el-radio-button label="error">错误</el-radio-button>
        <el-radio-button label="warning">警告</el-radio-button>
        <el-radio-button label="info">提示</el-radio-button>
      </el-radio-group>
      
      <el-input
        v-model="searchText"
        placeholder="搜索检查项..."
        prefix-icon="Search"
        clearable
        style="width: 200px; margin-left: 16px"
      />
      
      <el-button type="primary" @click="exportReport" style="margin-left: auto">
        <el-icon><Download /></el-icon>
        导出报告
      </el-button>
    </div>
    
    <!-- 结果列表 -->
    <div class="result-list">
      <div
        v-for="(result, index) in filteredResults"
        :key="index"
        class="result-item"
        :class="result.status"
        @click="showDetail(result)"
      >
        <div class="result-header">
          <el-tag :type="getTagType(result.status)" size="small">
            {{ getStatusText(result.status) }}
          </el-tag>
          <span class="rule-name">{{ result.rule_name }}</span>
          <el-button link @click.stop="locateInDocument(result)">
            <el-icon><Location /></el-icon>
            定位
          </el-button>
        </div>
        
        <div class="result-content">
          <p class="description">{{ result.description }}</p>
          <p v-if="result.suggestion" class="suggestion">
            <el-icon><InfoFilled /></el-icon>
            {{ result.suggestion }}
          </p>
        </div>
        
        <!-- 对比视图 -->
        <div v-if="result.expected && result.actual" class="compare-view">
          <div class="compare-item">
            <div class="compare-label">招标要求</div>
            <div class="compare-value expected">{{ result.expected }}</div>
          </div>
          <div class="compare-item">
            <div class="compare-label">投标响应</div>
            <div class="compare-value actual" :class="{ mismatch: result.expected !== result.actual }">
              {{ result.actual }}
            </div>
          </div>
        </div>
      </div>
    </div>
    
    <!-- 详情弹窗 -->
    <el-dialog v-model="showDetailDialog" :title="currentResult?.rule_name" width="600px">
      <div v-if="currentResult" class="detail-content">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="状态">
            <el-tag :type="getTagType(currentResult.status)">
              {{ getStatusText(currentResult.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="描述">
            {{ currentResult.description }}
          </el-descriptions-item>
          <el-descriptions-item label="建议">
            {{ currentResult.suggestion || '无' }}
          </el-descriptions-item>
          <el-descriptions-item label="位置" v-if="currentResult.location">
            第 {{ currentResult.location.page }} 页，第 {{ currentResult.location.line }} 行
          </el-descriptions-item>
        </el-descriptions>
        
        <div v-if="currentResult.context" class="context-preview">
          <h4>上下文</h4>
          <div class="context-text" v-html="highlightContext(currentResult)"></div>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="locateInDocument(currentResult)">
          <el-icon><Location /></el-icon>
          定位到文档
        </el-button>
        <el-button type="primary" @click="showDetailDialog = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { useAppStore } from '@/stores'
import { ElMessage } from 'element-plus'
import { Download, Location, InfoFilled } from '@element-plus/icons-vue'

const route = useRoute()
const appStore = useAppStore()

const filter = ref('all')
const searchText = ref('')
const showDetailDialog = ref(false)
const currentResult = ref(null)

// 计算属性
const results = computed(() => appStore.project.checkResults || [])

const filteredResults = computed(() => {
  let filtered = results.value
  
  // 状态筛选
  if (filter.value !== 'all') {
    filtered = filtered.filter(r => r.status === filter.value)
  }
  
  // 搜索筛选
  if (searchText.value) {
    const search = searchText.value.toLowerCase()
    filtered = filtered.filter(r => 
      r.rule_name.toLowerCase().includes(search) ||
      r.description?.toLowerCase().includes(search)
    )
  }
  
  return filtered
})

const errorCount = computed(() => results.value.filter(r => r.status === 'error').length)
const warningCount = computed(() => results.value.filter(r => r.status === 'warning').length)
const infoCount = computed(() => results.value.filter(r => r.status === 'info').length)

const score = computed(() => {
  const total = results.value.length
  if (total === 0) return 100
  
  const errorPenalty = errorCount.value * 10
  const warningPenalty = warningCount.value * 5
  const infoPenalty = infoCount.value * 2
  
  return Math.max(0, 100 - errorPenalty - warningPenalty - infoPenalty)
})

// 方法
function getTagType(status) {
  const types = {
    error: 'danger',
    warning: 'warning',
    info: 'info'
  }
  return types[status] || 'info'
}

function getStatusText(status) {
  const texts = {
    error: '错误',
    warning: '警告',
    info: '提示'
  }
  return texts[status] || status
}

function showDetail(result) {
  currentResult.value = result
  showDetailDialog.value = true
}

function locateInDocument(result) {
  if (result.location) {
    // TODO: 调用后端定位到文档位置
    ElMessage.info(`定位到第${result.location.page}页第${result.location.line}行`)
  } else {
    ElMessage.warning('该结果没有位置信息')
  }
}

function highlightContext(result) {
  if (!result.context || !result.highlight) return result.context
  
  return result.context.replace(
    result.highlight,
    `<mark class="highlight">${result.highlight}</mark>`
  )
}

async function exportReport() {
  const options = {
    watermark: true,
    desensitize: true,
    format: 'word'
  }
  
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke('export_report', {
      project: appStore.project,
      results: results.value,
      outputPath: `${appStore.project.name}_检查报告_${Date.now()}.html`,
      options
    })
    
    ElMessage.success(`报告已导出: ${result.path}`)
  } catch (error) {
    ElMessage.error(`导出失败: ${error}`)
  }
}
</script>

<style scoped>
.result-page {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

/* 统计卡片 */
.stats-cards {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  padding: 24px;
  border-radius: 12px;
  text-align: center;
  background: var(--card-bg);
  border: 1px solid var(--border-color);
}

.stat-card.error {
  border-left: 4px solid #F56C6C;
  background: linear-gradient(135deg, #FEF0F0 0%, var(--card-bg) 100%);
}

.stat-card.warning {
  border-left: 4px solid #E6A23C;
  background: linear-gradient(135deg, #FDF6EC 0%, var(--card-bg) 100%);
}

.stat-card.info {
  border-left: 4px solid #909399;
  background: linear-gradient(135deg, #F4F4F5 0%, var(--card-bg) 100%);
}

.stat-card.score {
  border-left: 4px solid #67C23A;
  background: linear-gradient(135deg, #F0F9EB 0%, var(--card-bg) 100%);
}

.stat-number {
  font-size: 36px;
  font-weight: bold;
  color: var(--text-primary);
}

.stat-label {
  font-size: 14px;
  color: var(--text-secondary);
  margin-top: 4px;
}

/* 筛选栏 */
.filter-bar {
  display: flex;
  align-items: center;
  margin-bottom: 20px;
  padding: 16px;
  background: var(--card-bg);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

/* 结果列表 */
.result-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.result-item {
  padding: 16px;
  background: var(--card-bg);
  border-radius: 8px;
  border: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.3s;
}

.result-item:hover {
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
  transform: translateY(-2px);
}

.result-item.error {
  border-left: 4px solid #F56C6C;
}

.result-item.warning {
  border-left: 4px solid #E6A23C;
}

.result-item.info {
  border-left: 4px solid #909399;
}

.result-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
}

.rule-name {
  flex: 1;
  font-weight: 500;
  color: var(--text-primary);
}

.result-content {
  margin-bottom: 12px;
}

.description {
  color: var(--text-primary);
  margin-bottom: 8px;
}

.suggestion {
  color: var(--text-secondary);
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 对比视图 */
.compare-view {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
  margin-top: 12px;
}

.compare-item {
  padding: 8px;
}

.compare-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.compare-value {
  padding: 8px;
  background: var(--card-bg);
  border-radius: 4px;
}

.compare-value.mismatch {
  background: #FEF0F0;
  color: #F56C6C;
}

/* 详情弹窗 */
.detail-content {
  padding: 16px 0;
}

.context-preview {
  margin-top: 16px;
}

.context-text {
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 6px;
  font-family: monospace;
  white-space: pre-wrap;
}

.highlight {
  background: #FEF0F0;
  color: #F56C6C;
  padding: 2px 4px;
  border-radius: 2px;
}

/* 响应式 */
@media (max-width: 768px) {
  .stats-cards {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .filter-bar {
    flex-wrap: wrap;
    gap: 12px;
  }
  
  .compare-view {
    grid-template-columns: 1fr;
  }
}
</style>
