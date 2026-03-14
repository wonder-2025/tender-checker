<template>
  <div class="app-container" :class="{ 'dark-theme': isDarkTheme }">
    <!-- 侧边栏 -->
    <aside class="sidebar" v-if="!isMobile">
      <div class="logo">
        <el-icon><Document /></el-icon>
        <span>标书检查</span>
      </div>
      
      <el-menu
        :default-active="currentRoute"
        router
        class="sidebar-menu"
      >
        <el-menu-item index="/">
          <el-icon><HomeFilled /></el-icon>
          <span>首页</span>
        </el-menu-item>
        
        <el-menu-item index="/tender-extract">
          <el-icon><Document /></el-icon>
          <span>招标提取</span>
        </el-menu-item>
        
        <el-menu-item index="/checking">
          <el-icon><Search /></el-icon>
          <span>智能检查</span>
        </el-menu-item>
        
        <el-menu-item index="/result">
          <el-icon><List /></el-icon>
          <span>检查结果</span>
        </el-menu-item>
        
        <el-menu-item index="/history">
          <el-icon><Clock /></el-icon>
          <span>历史记录</span>
        </el-menu-item>
        
        <el-menu-item index="/settings">
          <el-icon><Setting /></el-icon>
          <span>设置</span>
        </el-menu-item>
      </el-menu>
      
      <!-- 快捷键提示 -->
      <div class="shortcut-tips">
        <div class="tip-item">
          <span>打开文件</span>
          <span class="shortcut-key">Ctrl+O</span>
        </div>
        <div class="tip-item">
          <span>开始检查</span>
          <span class="shortcut-key">Ctrl+E</span>
        </div>
        <div class="tip-item">
          <span>切换主题</span>
          <span class="shortcut-key">Ctrl+D</span>
        </div>
      </div>
    </aside>
    
    <!-- 移动端导航 -->
    <nav class="mobile-nav" v-if="isMobile">
      <el-menu
        :default-active="currentRoute"
        router
        mode="horizontal"
        class="mobile-menu"
      >
        <el-menu-item index="/">
          <el-icon><HomeFilled /></el-icon>
        </el-menu-item>
        <el-menu-item index="/checking">
          <el-icon><Search /></el-icon>
        </el-menu-item>
        <el-menu-item index="/result">
          <el-icon><List /></el-icon>
        </el-menu-item>
        <el-menu-item index="/settings">
          <el-icon><Setting /></el-icon>
        </el-menu-item>
      </el-menu>
    </nav>
    
    <!-- 主内容 -->
    <main class="main-content">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
    
    <!-- 快捷键帮助弹窗 -->
    <el-dialog 
      v-model="showShortcutHelp" 
      title="快捷键" 
      width="400px"
      center
    >
      <div class="shortcut-list">
        <div v-for="shortcut in shortcuts" :key="shortcut.key" class="shortcut-item">
          <span>{{ shortcut.action }}</span>
          <span class="shortcut-key">{{ shortcut.key }}</span>
        </div>
      </div>
    </el-dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { 
  Document, HomeFilled, Search, List, Clock, Setting 
} from '@element-plus/icons-vue'

const route = useRoute()
const currentRoute = computed(() => route.path)
const isDarkTheme = ref(false)
const isMobile = ref(false)
const showShortcutHelp = ref(false)

// 快捷键列表
const shortcuts = [
  { key: 'Ctrl+O', action: '打开文件' },
  { key: 'Ctrl+E', action: '开始检查' },
  { key: 'Ctrl+S', action: '保存结果' },
  { key: 'Ctrl+P', action: '导出报告' },
  { key: 'Ctrl+D', action: '切换暗色主题' },
  { key: 'Ctrl+/', action: '显示快捷键帮助' },
  { key: 'Esc', action: '取消操作' },
]

// 键盘事件处理
function handleKeydown(e) {
  // Ctrl + O: 打开文件
  if (e.ctrlKey && e.key === 'o') {
    e.preventDefault()
    const uploadArea = document.querySelector('.upload-area')
    uploadArea?.click()
  }
  
  // Ctrl + E: 开始检查
  if (e.ctrlKey && e.key === 'e') {
    e.preventDefault()
    const startBtn = document.querySelector('.start-check-btn')
    startBtn?.click()
  }
  
  // Ctrl + D: 切换主题
  if (e.ctrlKey && e.key === 'd') {
    e.preventDefault()
    toggleTheme()
  }
  
  // Ctrl + /: 显示快捷键帮助
  if (e.ctrlKey && e.key === '/') {
    e.preventDefault()
    showShortcutHelp.value = true
  }
  
  // Esc: 关闭弹窗
  if (e.key === 'Escape') {
    showShortcutHelp.value = false
  }
}

// 切换主题
function toggleTheme() {
  isDarkTheme.value = !isDarkTheme.value
  localStorage.setItem('theme', isDarkTheme.value ? 'dark' : 'light')
  ElMessage.success(isDarkTheme.value ? '已切换到暗色主题' : '已切换到亮色主题')
}

// 检查屏幕宽度
function checkMobile() {
  isMobile.value = window.innerWidth < 768
}

// 初始化
onMounted(() => {
  // 检查主题
  const savedTheme = localStorage.getItem('theme')
  if (savedTheme === 'dark' || 
      (!savedTheme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
    isDarkTheme.value = true
  }
  
  // 监听键盘事件
  window.addEventListener('keydown', handleKeydown)
  
  // 监听窗口大小变化
  window.addEventListener('resize', checkMobile)
  checkMobile()
  
  // 首次使用提示
  const hasUsedBefore = localStorage.getItem('hasUsedBefore')
  if (!hasUsedBefore) {
    setTimeout(() => {
      ElMessage.info('欢迎使用标书智能检查工具！')
      localStorage.setItem('hasUsedBefore', 'true')
    }, 1000)
  }
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  window.removeEventListener('resize', checkMobile)
})
</script>

<style scoped>
.app-container {
  display: flex;
  min-height: 100vh;
  background: var(--bg-primary);
  transition: background-color 0.3s;
}

/* 侧边栏 */
.sidebar {
  width: 220px;
  background: var(--card-bg);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  position: fixed;
  left: 0;
  top: 0;
  bottom: 0;
  z-index: 100;
  transition: all 0.3s;
}

.logo {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px;
  font-size: 18px;
  font-weight: bold;
  color: var(--primary-color);
  border-bottom: 1px solid var(--border-color);
}

.logo .el-icon {
  font-size: 28px;
}

.sidebar-menu {
  flex: 1;
  border: none;
  background: transparent;
}

.sidebar-menu .el-menu-item {
  height: 50px;
  line-height: 50px;
  margin: 4px 12px;
  border-radius: 8px;
  color: var(--text-secondary);
  transition: all 0.3s;
}

.sidebar-menu .el-menu-item:hover {
  background: var(--bg-secondary);
  color: var(--primary-color);
}

.sidebar-menu .el-menu-item.is-active {
  background: rgba(64, 158, 255, 0.1);
  color: var(--primary-color);
  font-weight: 500;
}

/* 快捷键提示 */
.shortcut-tips {
  padding: 16px;
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  color: var(--text-tertiary);
}

.tip-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 0;
}

/* 移动端导航 */
.mobile-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  background: var(--card-bg);
  border-top: 1px solid var(--border-color);
  z-index: 100;
}

.mobile-menu {
  display: flex;
  justify-content: space-around;
  border: none;
  background: transparent;
}

.mobile-menu .el-menu-item {
  flex: 1;
  text-align: center;
}

/* 主内容 */
.main-content {
  flex: 1;
  margin-left: 220px;
  padding: 20px;
  min-height: 100vh;
  transition: margin-left 0.3s;
}

/* 快捷键列表 */
.shortcut-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.shortcut-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-color);
}

.shortcut-item:last-child {
  border-bottom: none;
}

/* 响应式 */
@media (max-width: 768px) {
  .sidebar {
    display: none;
  }
  
  .main-content {
    margin-left: 0;
    padding-bottom: 60px;
  }
}
</style>
