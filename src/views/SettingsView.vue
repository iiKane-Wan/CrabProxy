<script setup lang="ts">
import { useSettingsStore } from '../stores/settingsStore'
import { useMessage } from 'naive-ui'
import type { ThemeMode } from '../types'

const settingsStore = useSettingsStore()
const message = useMessage()

const themeOptions = [
  { label: '浅色', value: 'light' as ThemeMode },
  { label: '深色', value: 'dark' as ThemeMode },
  { label: '跟随系统', value: 'system' as ThemeMode },
]

onMounted(async () => {
  await settingsStore.fetchStartup()
})

async function handleStartupChange(enabled: boolean) {
  try {
    await settingsStore.setStartup(enabled)
    message.success(enabled ? '已开启开机自启' : '已关闭开机自启')
  } catch (e: any) {
    message.error(`设置失败: ${e}`)
  }
}

function handleThemeChange(theme: ThemeMode) {
  settingsStore.setTheme(theme)
}
</script>

<template>
  <div>
    <h2 style="margin-top: 0; margin-bottom: 16px">设置</h2>

    <n-space vertical :size="16">
      <!-- 开机自启 -->
      <n-card size="small" title="开机自启">
        <n-space align="center" justify="space-between">
          <span>系统启动时自动运行端口代理工具</span>
          <n-switch
            :value="settingsStore.startupEnabled"
            @update:value="handleStartupChange"
          />
        </n-space>
      </n-card>

      <!-- 主题选择 -->
      <n-card size="small" title="主题设置">
        <n-radio-group :value="settingsStore.theme" @update:value="handleThemeChange">
          <n-space>
            <n-radio v-for="opt in themeOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </n-radio>
          </n-space>
        </n-radio-group>
      </n-card>

      <!-- 关于 -->
      <n-card size="small" title="关于">
        <n-space vertical :size="4">
          <n-text>CrabProxy v0.1.0</n-text>
          <n-text depth="3">
            基于 Rust + Tauri + Vue3 构建的跨平台端口代理工具
          </n-text>
          <n-text depth="3"> 配置文件存储目录由系统自动管理，无需手动干预 </n-text>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>
