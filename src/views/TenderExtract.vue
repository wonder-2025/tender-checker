<template>
  <div class="tender-extract-page">
    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>招标文件提取结果</span>
          <div class="header-actions">
            <el-button @click="reExtract">
              <el-icon><RefreshRight /></el-icon>
              重新提取
            </el-button>
            <el-button type="primary" @click="confirmExtraction">
              确认使用
            </el-button>
          </div>
        </div>
      </template>
      
      <!-- 项目信息 -->
      <div class="project-info">
        <h4>项目信息</h4>
        <el-descriptions :column="3" border>
          <el-descriptions-item label="项目名称">
            {{ extraction.projectInfo?.projectName || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="项目编号">
            {{ extraction.projectInfo?.projectNo || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="招标单位">
            {{ extraction.projectInfo?.tenderingUnit || '-' }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
      
      <!-- 评分表 -->
      <div class="section">
        <div class="section-header">
          <h4>📊 评分表 ({{ extraction.scoringTable?.length || 0 }}项)</h4>
          <el-button type="primary" link @click="editScoringTable">编辑</el-button>
        </div>
        
        <el-table :data="extraction.scoringTable" style="width: 100%" max-height="400">
          <el-table-column prop="id" label="序号" width="80" />
          <el-table-column prop="category" label="类别" width="120" />
          <el-table-column prop="item" label="评审项" />
          <el-table-column prop="score" label="分值" width="80" />
          <el-table-column prop="requirement" label="要求" show-overflow-tooltip />
        </el-table>
      </div>
      
      <!-- 必填章节 -->
      <div class="section">
        <div class="section-header">
          <h4>📋 必填章节 ({{ extraction.requiredSections?.length || 0 }}项)</h4>
          <el-button type="primary" link @click="editRequiredSections">编辑</el-button>
        </div>
        
        <div class="sections-grid">
          <el-checkbox 
            v-for="section in extraction.requiredSections" 
            :key="section.name"
            :label="section.name"
            :model-value="section.required"
            disabled
          />
        </div>
      </div>
      
      <!-- 资质要求 -->
      <div class="section">
        <div class="section-header">
          <h4>🏢 资质要求 ({{ extraction.qualificationRequirements?.length || 0 }}项)</h4>
          <el-button type="primary" link @click="editQualificationRequirements">编辑</el-button>
        </div>
        
        <div class="requirements-grid">
          <div 
            v-for="req in extraction.qualificationRequirements" 
            :key="req.name"
            class="requirement-item"
          >
            <el-icon><CircleCheck /></el-icon>
            <span>{{ req.name }}</span>
          </div>
        </div>
      </div>
      
      <!-- 时间要求 -->
      <div class="section">
        <div class="section-header">
          <h4>⏰ 时间要求</h4>
          <el-button type="primary" link @click="editTimeRequirements">编辑</el-button>
        </div>
        
        <el-descriptions :column="3" border>
          <el-descriptions-item label="工期要求">
            {{ extraction.timeRequirements?.projectPeriod || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="投标有效期">
            {{ extraction.timeRequirements?.bidValidity || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="投标保证金">
            {{ extraction.timeRequirements?.bidBond?.amount || '-' }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
      
      <!-- 格式要求 -->
      <div class="section">
        <div class="section-header">
          <h4>📝 格式要求</h4>
          <el-button type="primary" link @click="editFormatRequirements">编辑</el-button>
        </div>
        
        <el-descriptions :column="2" border>
          <el-descriptions-item label="正文字体">
            {{ extraction.formatRequirements?.bodyFont?.name || '-' }} / 
            {{ extraction.formatRequirements?.bodyFont?.size || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="标题字体">
            {{ extraction.formatRequirements?.titleFont?.name || '-' }} / 
            {{ extraction.formatRequirements?.titleFont?.size || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="页边距">
            上: {{ extraction.formatRequirements?.pageMargin?.top || '-' }}、
            下: {{ extraction.formatRequirements?.pageMargin?.bottom || '-' }}、
            左: {{ extraction.formatRequirements?.pageMargin?.left || '-' }}、
            右: {{ extraction.formatRequirements?.pageMargin?.right || '-' }}
          </el-descriptions-item>
          <el-descriptions-item label="行间距">
            {{ extraction.formatRequirements?.lineSpacing || '-' }}
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </el-card>
    
    <!-- 编辑对话框 -->
    <el-dialog v-model="editDialogVisible" :title="editDialogTitle" width="600px">
      <component :is="currentEditComponent" v-model="currentEditData" />
      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="saveEdit">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, reactive, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores'
import { ElMessage } from 'element-plus'
import { RefreshRight, CircleCheck } from '@element-plus/icons-vue'
import { invoke } from '@tauri-apps/api'

const router = useRouter()
const appStore = useAppStore()

const extraction = computed(() => appStore.project.tenderExtraction || {
  projectInfo: {},
  scoringTable: [],
  requiredSections: [],
  qualificationRequirements: [],
  timeRequirements: {},
  formatRequirements: {}
})

const editDialogVisible = ref(false)
const editDialogTitle = ref('')
const currentEditComponent = ref(null)
const currentEditData = ref({})

function reExtract() {
  // TODO: 重新调用AI提取
  ElMessage.success('重新提取中...')
}

function confirmExtraction() {
  ElMessage.success('已确认使用提取结果')
  router.push('/')
}

function editScoringTable() {
  editDialogTitle.value = '编辑评分表'
  currentEditData.value = [...extraction.value.scoringTable]
  editDialogVisible.value = true
}

function editRequiredSections() {
  editDialogTitle.value = '编辑必填章节'
  currentEditData.value = [...extraction.value.requiredSections]
  editDialogVisible.value = true
}

function editQualificationRequirements() {
  editDialogTitle.value = '编辑资质要求'
  currentEditData.value = [...extraction.value.qualificationRequirements]
  editDialogVisible.value = true
}

function editTimeRequirements() {
  editDialogTitle.value = '编辑时间要求'
  currentEditData.value = { ...extraction.value.timeRequirements }
  editDialogVisible.value = true
}

function editFormatRequirements() {
  editDialogTitle.value = '编辑格式要求'
  currentEditData.value = { ...extraction.value.formatRequirements }
  editDialogVisible.value = true
}

function saveEdit() {
  // TODO: 保存编辑结果
  ElMessage.success('已保存')
  editDialogVisible.value = false
}
</script>

<style scoped>
.tender-extract-page {
  max-width: 1000px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 12px;
}

.project-info {
  margin-bottom: 24px;
}

.project-info h4 {
  margin-bottom: 12px;
}

.section {
  margin-bottom: 24px;
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

.sections-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.requirements-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 12px;
}

.requirement-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: #f0f9eb;
  border-radius: 8px;
  color: #67c23a;
}
</style>
