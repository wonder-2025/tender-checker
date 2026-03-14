<template>
  <div class="history-page">
    <!-- 搜索和筛选 -->
    <div class="toolbar">
      <el-input
        v-model="searchText"
        placeholder="搜索项目名称..."
        prefix-icon="Search"
        clearable
        style="width: 300px"
      />
      
      <el-date-picker
        v-model="dateRange"
        type="daterange"
        range-separator="至"
        start-placeholder="开始日期"
        end-placeholder="结束日期"
        style="width: 300px"
      />
      
      <el-button type="primary" @click="showCompareDialog = true" :disabled="selectedItems.length < 2">
        <el-icon><DataAnalysis /></el-icon>
        对比选中 ({{ selectedItems.length }})
      </el-button>
    </div>
    
    <!-- 历史列表 -->
    <div class="history-list">
      <el-table 
        :data="filteredHistory" 
        style="width: 100%"
        @selection-change="handleSelectionChange"
        v-loading="loading"
      >
        <el-table-column type="selection" width="55" />
        
        <el-table-column prop="name" label="项目名称" min-width="200">
          <template #default="{ row }">
            <div class="project-name">
              <el-icon><Document /></el-icon>
              <span>{{ row.name }}</span>
            </div>
          </template>
        </el-table-column>
        
        <el-table-column prop="checkTime" label="检查时间" width="180" sortable>
          <template #default="{ row }">
            {{ formatDate(row.checkTime) }}
          </template>
        </el-table-column>
        
        <el-table-column prop="status" label="状态" width="100" align="center">
          <template #default="{ row }">
            <el-tag :type="row.status === 'completed' ? 'success' : 'info'">
              {{ row.status === 'completed' ? '已完成' : '进行中' }}
            </el-tag>
          </template>
        </el-table-column>
        
        <el-table-column label="检查结果" width="280">
          <template #default="{ row }">
            <div class="result-stats">
              <span class="error">{{ row.errorCount }} 错误</span>
              <span class="warning">{{ row.warningCount }} 警告</span>
              <span class="info">{{ row.infoCount }} 提示</span>
            </div>
          </template>
        </el-table-column>
        
        <el-table-column prop="score" label="评分" width="100" align="center" sortable>
          <template #default="{ row }">
            <el-tag :type="getScoreType(row.score)">
              {{ row.score }}分
            </el-tag>
          </template>
        </el-table-column>
        
        <el-table-column label="操作" width="200" fixed="right">
          <template #default="{ row }">
            <el-button link type="primary" @click="viewResult(row)">
              查看结果
            </el-button>
            <el-button link type="warning" @click="recheck(row)">
              重新检查
            </el-button>
            <el-button link type="danger" @click="deleteRecord(row)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>
    
    <!-- 分页 -->
    <div class="pagination">
      <el-pagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :page-sizes="[10, 20, 50, 100]"
        :total="totalRecords"
        layout="total, sizes, prev, pager, next, jumper"
        @size-change="loadHistory"
        @current-change="loadHistory"
      />
    </div>
    
    <!-- 对比弹窗 -->
    <el-dialog 
      v-model="showCompareDialog" 
      title="项目对比" 
      width="80%"
      :close-on-click-modal="false"
    >
      <div class="compare-container">
        <!-- 对比统计 -->
        <div class="compare-stats">
          <div class="stat-row header">
            <div class="stat-label">项目</div>
            <div v-for="item in selectedItems" :key="item.id" class="stat-value">
              {{ item.name }}
            </div>
          </div>
          
          <div class="stat-row">
            <div class="stat-label">评分</div>
            <div v-for="item in selectedItems" :key="item.id" class="stat-value">
              <el-tag :type="getScoreType(item.score)">{{ item.score }}分</el-tag>
            </div>
          </div>
          
          <div class="stat-row">
            <div class="stat-label">错误</div>
            <div v-for="item in selectedItems" :key="item.id" class="stat-value error">
              {{ item.errorCount }}
            </div>
          </div>
          
          <div class="stat-row">
            <div class="stat-label">警告</div>
            <div v-for="item in selectedItems" :key="item.id" class="stat-value warning">
              {{ item.warningCount }}
            </div>
          </div>
          
          <div class="stat-row">
            <div class="stat-label">提示</div>
            <div v-for="item in selectedItems" :key="item.id" class="stat-value info">
              {{ item.infoCount }}
            </div>
          </div>
        </div>
        
        <!-- 对比详情 -->
        <el-divider content-position="left">详细对比</el-divider>
        
        <div class="compare-details">
          <el-table :data="compareDetails" border>
            <el-table-column prop="checkItem" label="检查项" width="200" fixed />
            <el-table-column v-for="item in selectedItems" :key="item.id" :label="item.name">
              <template #default="{ row }">
                <el-tag 
                  :type="getTagType(row.results[item.id]?.status)"
                  size="small"
                >
                  {{ row.results[item.id]?.text || '-' }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </div>
      </div>
      
      <template #footer>
        <el-button @click="showCompareDialog = false">关闭</el-button>
        <el-button type="primary" @click="exportComparison">
          <el-icon><Download /></el-icon>
          导出对比报告
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Document, DataAnalysis, Download } from '@element-plus/icons-vue'

const router = useRouter()

const loading = ref(false)
const searchText = ref('')
const dateRange = ref([])
const selectedItems = ref([])
const showCompareDialog = ref(false)
const compareDetails = ref([])
const currentPage = ref(1)
const pageSize = ref(10)
const totalRecords = ref(0)

// 模拟历史数据
const historyData = ref([
  {
    id: 1,
    name: 'XX市政工程项目投标文件',
    checkTime: new Date('2024-01-15 10:30:00'),
    status: 'completed',
    errorCount: 2,
    warningCount: 5,
    infoCount: 3,
    score: 75
  },
  {
    id: 2,
    name: 'YY智能化系统采购项目',
    checkTime: new Date('2024-01-14 15:20:00'),
    status: 'completed',
    errorCount: 0,
    warningCount: 2,
    infoCount: 8,
    score: 90
  },
  {
    id: 3,
    name: 'ZZ办公楼装修工程',
    checkTime: new Date('2024-01-13 09:15:00'),
    status: 'completed',
    errorCount: 5,
    warningCount: 10,
    infoCount: 6,
    score: 55
  }
])

const filteredHistory = computed(() => {
  let filtered = historyData.value
  
  if (searchText.value) {
    const search = searchText.value.toLowerCase()
    filtered = filtered.filter(h => h.name.toLowerCase().includes(search))
  }
  
  if (dateRange.value && dateRange.value.length === 2) {
    const [start, end] = dateRange.value
    filtered = filtered.filter(h => {
      const time = new Date(h.checkTime)
      return time >= start && time <= end
    })
  }
  
  return filtered
})

function formatDate(date) {
  return new Date(date).toLocaleString('zh-CN')
}

function getScoreType(score) {
  if (score >= 80) return 'success'
  if (score >= 60) return 'warning'
  return 'danger'
}

function getTagType(status) {
  const types = { success: 'success', error: 'danger', warning: 'warning', info: 'info' }
  return types[status] || 'info'
}

function handleSelectionChange(selection) {
  selectedItems.value = selection
}

async function loadHistory() {
  loading.value = true
  totalRecords.value = historyData.value.length
  loading.value = false
}

function viewResult(row) {
  // TODO: 加载历史结果并跳转
  router.push('/result')
}

function recheck(row) {
  ElMessageBox.confirm(`确定要重新检查"${row.name}"吗？`, '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'info'
  }).then(() => {
    router.push('/checking')
  }).catch(() => {})
}

async function deleteRecord(row) {
  try {
    await ElMessageBox.confirm(`确定要删除"${row.name}"的记录吗？`, '警告', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })
    
    const index = historyData.value.findIndex(h => h.id === row.id)
    if (index > -1) {
      historyData.value.splice(index, 1)
      ElMessage.success('删除成功')
    }
  } catch (error) {
    // 用户取消
  }
}

function exportComparison() {
  ElMessage.info('导出对比报告功能开发中...')
}

onMounted(() => {
  loadHistory()
})
</script>

<style scoped>
.history-page {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.toolbar {
  display: flex;
  gap: 16px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.history-list {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 20px;
  border: 1px solid var(--border-color);
}

.project-name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.result-stats {
  display: flex;
  gap: 12px;
  font-size: 14px;
}

.result-stats .error { color: #F56C6C; }
.result-stats .warning { color: #E6A23C; }
.result-stats .info { color: #909399; }

.pagination {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}

/* 对比弹窗 */
.compare-container {
  padding: 20px;
}

.compare-stats {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
}

.stat-row {
  display: grid;
  grid-template-columns: 200px repeat(var(--cols), 1fr);
  border-bottom: 1px solid var(--border-color);
}

.stat-row:last-child {
  border-bottom: none;
}

.stat-row.header {
  background: var(--bg-secondary);
  font-weight: bold;
}

.stat-label, .stat-value {
  padding: 12px 16px;
  border-right: 1px solid var(--border-color);
}

.stat-label {
  background: var(--bg-secondary);
  font-weight: 500;
}

.stat-value.error { color: #F56C6C; font-weight: bold; }
.stat-value.warning { color: #E6A23C; font-weight: bold; }
.stat-value.info { color: #909399; }

.compare-details {
  margin-top: 20px;
}

/* 响应式 */
@media (max-width: 768px) {
  .toolbar {
    flex-direction: column;
  }
  
  .toolbar > * {
    width: 100% !important;
  }
  
  .stat-row {
    display: block;
  }
  
  .stat-label, .stat-value {
    display: inline-block;
    width: 50%;
    box-sizing: border-box;
  }
}
</style>
