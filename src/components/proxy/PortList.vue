<script setup lang="ts">
import type { PortState } from '../../types'

interface Props {
  ports: PortState[]
  editable?: boolean
  loading?: boolean
}

withDefaults(defineProps<Props>(), {
  editable: false,
  loading: false,
})

const columns = [
  { title: '本地端口', key: 'local_port', width: 100 },
  { title: '目标 IP', key: 'target_ip' },
  { title: '状态', key: 'status', width: 140 },
  { title: '操作', key: 'actions', width: 200 },
]
</script>

<template>
  <n-data-table
    :columns="columns"
    :data="ports"
    :loading="loading"
    :bordered="false"
    :single-line="false"
    size="small"
  >
    <template #empty>
      <n-empty description="暂无端口配置" />
    </template>
  </n-data-table>
</template>
