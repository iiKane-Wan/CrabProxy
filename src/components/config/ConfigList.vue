<script setup lang="ts">
import type { ConfigMeta } from '../../types'

interface Props {
  configs: ConfigMeta[]
  activeName: string | null
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  loading: false,
})

const emit = defineEmits<{
  select: [name: string]
  edit: [config: ConfigMeta]
  delete: [name: string]
}>()

const columns = [
  { title: '配置名称', key: 'name' },
  { title: '全局 IP', key: 'global_ip', width: 160 },
  { title: '端口数', key: 'port_count', width: 80 },
  { title: '已启用', key: 'enabled_count', width: 80 },
]
</script>

<template>
  <n-data-table
    :columns="columns"
    :data="configs"
    :loading="loading"
    :bordered="false"
    size="small"
  >
    <template #empty>
      <n-empty description="暂无配置文件，请点击下方按钮创建" />
    </template>
  </n-data-table>
</template>
