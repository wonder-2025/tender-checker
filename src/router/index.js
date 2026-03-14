import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue'),
    meta: { title: '首页' }
  },
  {
    path: '/tender-extract',
    name: 'TenderExtract',
    component: () => import('@/views/TenderExtract.vue'),
    meta: { title: '招标文件提取结果' }
  },
  {
    path: '/checking',
    name: 'Checking',
    component: () => import('@/views/Checking.vue'),
    meta: { title: '检查中' }
  },
  {
    path: '/result',
    name: 'Result',
    component: () => import('@/views/Result.vue'),
    meta: { title: '检查结果' }
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/Settings.vue'),
    meta: { title: '设置' }
  },
  {
    path: '/history',
    name: 'History',
    component: () => import('@/views/History.vue'),
    meta: { title: '历史记录' }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
