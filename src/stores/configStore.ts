import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { ConfigMeta, ProxyConfig } from '../types'

interface ConfigState {
  configs: ConfigMeta[]
  currentConfig: ProxyConfig | null
  isLoading: boolean
}

export const useConfigStore = defineStore('config', {
  state: (): ConfigState => ({
    configs: [],
    currentConfig: null,
    isLoading: false,
  }),

  actions: {
    /** 获取所有配置列表 */
    async fetchConfigs() {
      this.isLoading = true
      try {
        this.configs = await invoke<ConfigMeta[]>('get_all_configs')
      } catch (e) {
        console.error('获取配置列表失败:', e)
      } finally {
        this.isLoading = false
      }
    },

    /** 加载指定配置为当前激活配置 */
    async loadConfig(name: string) {
      try {
        this.currentConfig = await invoke<ProxyConfig>('load_config', { name })
      } catch (e) {
        console.error('加载配置失败:', e)
        throw e
      }
    },

    /** 切换激活配置并启动代理 */
    async switchConfig(name: string) {
      try {
        await invoke('switch_config', { name })
        await this.loadConfig(name)
      } catch (e) {
        console.error('切换配置失败:', e)
        throw e
      }
    },

    /** 保存配置（新建或更新） */
    async saveConfig(config: ProxyConfig) {
      try {
        await invoke('save_config', { config })
        await this.fetchConfigs()
      } catch (e) {
        console.error('保存配置失败:', e)
        throw e
      }
    },

    /** 删除配置 */
    async deleteConfig(name: string) {
      try {
        await invoke('delete_config', { name })
        // 如果删除的是当前配置，清空
        if (this.currentConfig?.name === name) {
          this.currentConfig = null
        }
        await this.fetchConfigs()
      } catch (e) {
        console.error('删除配置失败:', e)
        throw e
      }
    },

    /** 重命名配置 */
    async renameConfig(oldName: string, newName: string) {
      try {
        await invoke('rename_config', { oldName, newName })
        if (this.currentConfig?.name === oldName) {
          this.currentConfig!.name = newName
        }
        await this.fetchConfigs()
      } catch (e) {
        console.error('重命名配置失败:', e)
        throw e
      }
    },
  },
})
