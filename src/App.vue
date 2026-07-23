<script setup lang="ts">
import { darkTheme, useOsTheme } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import AppSidebar from './components/layout/AppSidebar.vue'
import { useSettingsStore } from './stores/settingsStore'
import type { ThemeMode } from './types'

const settingsStore = useSettingsStore()
const osTheme = useOsTheme()

// 判断当前是否深色模式
const isDark = computed(() => {
  const mode: ThemeMode = settingsStore.theme
  if (mode === 'dark') return true
  if (mode === 'light') return false
  return osTheme.value === 'dark'
})

// 根据用户选择和系统主题计算 Naive UI 主题
const actualTheme = computed(() => {
  return isDark.value ? darkTheme : null
})

// 同步窗口标题栏主题到 Rust 后端
watch(isDark, (dark) => {
  invoke('set_window_theme', { theme: dark ? 'dark' : 'light' }).catch(() => {
    // 命令在旧版 Tauri 可能不可用，静默忽略
  })
}, { immediate: true })

onMounted(() => {
  settingsStore.setTheme(settingsStore.theme)
})
</script>

<template>
  <n-config-provider :theme="actualTheme">
    <n-message-provider>
      <n-notification-provider>
        <n-dialog-provider>
          <n-layout has-sider style="height: 100vh">
            <AppSidebar />
            <n-layout>
              <n-layout-content
                style="padding: 24px; background: var(--n-color-body)"
              >
                <router-view />
              </n-layout-content>
            </n-layout>
          </n-layout>
        </n-dialog-provider>
      </n-notification-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
html,
body,
#app {
  margin: 0;
  padding: 0;
  height: 100%;
  overflow: hidden;
}
</style>
