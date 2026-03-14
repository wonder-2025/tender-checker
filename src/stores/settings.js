import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  // 错误日志上报配置
  const errorReport = ref({
    enabled: true,
    serverUrl: 'http://106.12.190.227:30051/api/tender-checker/error-log'
  })

  // 测试服务器连接
  const testConnection = async () => {
    try {
      const response = await fetch(errorReport.value.serverUrl.replace('/error-log', '/ping'), {
        method: 'GET',
        headers: { 'Content-Type': 'application/json' }
      })
      if (response.ok) {
        const data = await response.json()
        return { success: true, message: data.message || '连接成功' }
      }
      return { success: false, message: `HTTP ${response.status}` }
    } catch (error) {
      return { success: false, message: error.message }
    }
  }

  // 提交错误日志
  const submitErrorLog = async (errorData) => {
    if (!errorReport.value.enabled) {
      return { success: false, message: '错误日志上报已禁用' }
    }

    const payload = {
      app_name: 'tender-checker',
      version: '1.0.0',
      os: navigator.platform,
      ...errorData,
      timestamp: new Date().toISOString()
    }

    try {
      const response = await fetch(errorReport.value.serverUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload)
      })

      if (response.ok) {
        return { success: true, message: '错误日志已提交' }
      }
      return { success: false, message: `HTTP ${response.status}` }
    } catch (error) {
      return { success: false, message: error.message }
    }
  }

  // 从本地存储加载设置
  const loadSettings = () => {
    const saved = localStorage.getItem('tender-checker-settings')
    if (saved) {
      try {
        const settings = JSON.parse(saved)
        if (settings.errorReport) {
          errorReport.value = settings.errorReport
        }
      } catch (e) {
        console.error('加载设置失败:', e)
      }
    }
  }

  // 保存设置到本地存储
  watch([errorReport], () => {
    localStorage.setItem('tender-checker-settings', JSON.stringify({
      errorReport: errorReport.value
    }))
  }, { deep: true })

  return {
    errorReport,
    testConnection,
    submitErrorLog,
    loadSettings
  }
})
