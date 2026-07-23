import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import type { ThemeMode } from '../types'

interface SettingsState {
  startupEnabled: boolean
  theme: ThemeMode
}

export const useSettingsStore = defineStore('settings', {
  state: (): SettingsState => ({
    startupEnabled: false,
    theme: (localStorage.getItem('theme') as ThemeMode) || 'system',
  }),

  actions: {
    /** 获取开机自启状态 */
    async fetchStartup() {
      try {
        this.startupEnabled = await invoke<boolean>('get_startup')
      } catch (e) {
        console.error('获取开机自启状态失败:', e)
      }
    },

    /** 设置开机自启 */
    async setStartup(enabled: boolean) {
      try {
        await invoke('set_startup', { enabled })
        this.startupEnabled = enabled
      } catch (e) {
        console.error('设置开机自启失败:', e)
        throw e
      }
    },

    /** 设置主题 */
    setTheme(theme: ThemeMode) {
      this.theme = theme
      localStorage.setItem('theme', theme)
      this.applyTheme(theme)
    },

    /** 应用主题到 Naive UI */
    applyTheme(_theme: ThemeMode) {
      // Naive UI 主题切换通过 n-config-provider 的 theme 属性控制
      // 由 App.vue 中的 computed 响应
    },
  },
})
