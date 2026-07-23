<script setup lang="ts">
interface Props {
  globalEnabled: boolean
  loading: boolean
  activeConfigName: string | null
}

defineProps<Props>()

const emit = defineEmits<{
  'update:globalEnabled': [value: boolean]
}>()

function handleToggle(value: boolean) {
  emit('update:globalEnabled', value)
}
</script>

<template>
  <n-card size="small" style="margin-bottom: 16px">
    <n-space align="center" justify="space-between">
      <n-space align="center">
        <span style="font-weight: 500">代理状态</span>
        <n-tag v-if="activeConfigName" type="info" size="small">
          当前配置：{{ activeConfigName }}
        </n-tag>
        <n-tag v-else type="warning" size="small"> 未选择配置 </n-tag>
      </n-space>

      <n-space align="center">
        <n-spin v-if="loading" size="small" />
        <span style="font-size: 13px; color: var(--n-text-color-secondary)">
          {{ globalEnabled ? '代理运行中' : '代理已停止' }}
        </span>
        <n-switch
          :value="globalEnabled"
          :loading="loading"
          :disabled="!activeConfigName"
          @update:value="handleToggle"
        />
      </n-space>
    </n-space>
  </n-card>
</template>
