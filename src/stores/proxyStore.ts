import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { PortRule, ProxyState } from '../types'

interface ProxyStoreState {
  proxyState: ProxyState
  isLoading: boolean
}

export const useProxyStore = defineStore('proxy', {
  state: (): ProxyStoreState => ({
    proxyState: {
      active_config: null,
      global_enabled: false,
      ports: [],
    },
    isLoading: false,
  }),

  actions: {
    /** 初始化：获取代理状态并监听变化事件 */
    async init() {
      await this.fetchStatus()

      // 监听后端推送的状态变化
      await listen<ProxyState>('proxy-state-changed', (event) => {
        this.proxyState = event.payload
      })

      // 监听代理错误
      await listen<string>('proxy-error', (event) => {
        console.error('代理错误:', event.payload)
        // 错误发生时刷新状态
        this.fetchStatus()
      })
    },

    /** 获取当前代理状态 */
    async fetchStatus() {
      try {
        this.proxyState = await invoke<ProxyState>('get_proxy_status')
      } catch (e) {
        console.error('获取代理状态失败:', e)
      }
    },

    /** 启动时恢复上次会话的配置 */
    async restoreLastSession() {
      try {
        const name = await invoke<string | null>('restore_last_session')
        if (name) {
          // 恢复成功后刷新状态
          await this.fetchStatus()
        }
        return name
      } catch (e) {
        console.error('恢复上次会话失败:', e)
        return null
      }
    },

    /** 切换配置 */
    async switchConfig(name: string) {
      this.isLoading = true
      try {
        await invoke('switch_config', { name })
      } catch (e) {
        console.error('切换配置失败:', e)
        throw e
      } finally {
        this.isLoading = false
      }
    },

    /** 全局批量切换 */
    async toggleAllPorts(configName: string, enabled: boolean) {
      this.isLoading = true
      try {
        await invoke('toggle_all_ports', { configName, enabled })
      } catch (e) {
        console.error('批量切换失败:', e)
        throw e
      } finally {
        this.isLoading = false
      }
    },

    /** 添加端口 */
    async addPort(configName: string, port: PortRule) {
      try {
        await invoke('add_port', { configName, port })
      } catch (e) {
        console.error('添加端口失败:', e)
        throw e
      }
    },

    /** 更新端口 */
    async updatePort(configName: string, port: PortRule) {
      try {
        await invoke('update_port', { configName, port })
      } catch (e) {
        console.error('更新端口失败:', e)
        throw e
      }
    },

    /** 删除端口 */
    async removePort(configName: string, localPort: number) {
      try {
        await invoke('remove_port', { configName, localPort })
      } catch (e) {
        console.error('删除端口失败:', e)
        throw e
      }
    },
  },
})
