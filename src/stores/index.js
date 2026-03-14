import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export const useAppStore = defineStore('app', {
  state: () => ({
    // 项目信息
    project: {
      id: null,
      tenderFile: null,      // 招标文件
      bidFile: null,         // 投标文件
      tenderExtraction: null // 招标文件提取结果
    },
    
    // 检查配置
    checkConfig: {
      generalRules: [],      // 通用检查规则
      customRules: [],       // 自定义规则
      projectRules: []       // 项目特定规则
    },
    
    // API配置
    apiConfig: {
      provider: 'baidu',
      apiKey: '',
      model: 'ERNIE-4.0'
    },
    
    // 脱敏配置
    desensitizeRules: [],
    
    // 检查结果
    checkResult: null,
    
    // 检查进度
    checkProgress: {
      total: 0,
      current: 0,
      status: 'idle' // idle, running, completed, error
    }
  }),
  
  getters: {
    hasTenderFile: (state) => !!state.project.tenderFile,
    hasBidFile: (state) => !!state.project.bidFile,
    canStartCheck: (state) => !!state.project.bidFile && state.apiConfig.apiKey
  },
  
  actions: {
    // 设置招标文件
    async setTenderFile(file) {
      this.project.tenderFile = file
      // 自动提取招标文件信息
      if (file) {
        try {
          const result = await invoke('parse_tender_document', {
            filePath: file.path,
            llmConfig: this.apiConfig
          })
          this.project.tenderExtraction = result
          return result
        } catch (error) {
          console.error('招标文件解析失败:', error)
          throw error
        }
      }
    },
    
    // 设置投标文件
    setBidFile(file) {
      this.project.bidFile = file
    },
    
    // 设置API配置
    setApiConfig(config) {
      this.apiConfig = { ...this.apiConfig, ...config }
    },
    
    // 获取默认脱敏规则
    async loadDefaultDesensitizeRules() {
      try {
        const rules = await invoke('get_default_rules')
        this.desensitizeRules = rules
      } catch (error) {
        console.error('加载脱敏规则失败:', error)
      }
    },
    
    // 获取自定义检查规则
    async loadCustomRules() {
      try {
        const rules = await invoke('get_custom_check_rules')
        this.checkConfig.customRules = rules
      } catch (error) {
        console.error('加载自定义规则失败:', error)
      }
    },
    
    // 执行检查
    async executeCheck() {
      if (!this.canStartCheck) return
      
      this.checkProgress.status = 'running'
      this.checkProgress.current = 0
      
      try {
        const result = await invoke('execute_full_check', {
          projectId: this.project.id,
          bidFilePath: this.project.bidFile.path,
          tenderExtraction: this.project.tenderExtraction,
          generalRules: this.checkConfig.generalRules,
          customRules: this.checkConfig.customRules,
          desensitizeRules: this.desensitizeRules,
          llmConfig: this.apiConfig
        })
        
        this.checkResult = result
        this.checkProgress.status = 'completed'
        return result
      } catch (error) {
        this.checkProgress.status = 'error'
        throw error
      }
    },
    
    // 重置项目
    resetProject() {
      this.project = {
        id: null,
        tenderFile: null,
        bidFile: null,
        tenderExtraction: null
      }
      this.checkResult = null
      this.checkProgress = {
        total: 0,
        current: 0,
        status: 'idle'
      }
    }
  }
})
