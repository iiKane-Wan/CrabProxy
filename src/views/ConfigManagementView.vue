<script setup lang="ts">
import { useConfigStore } from '../stores/configStore'
import { useProxyStore } from '../stores/proxyStore'
import ConfigEditDialog from '../components/config/ConfigEditDialog.vue'
import ConfirmDialog from '../components/common/ConfirmDialog.vue'
import { useMessage } from 'naive-ui'
import type { ConfigMeta } from '../types'

const configStore = useConfigStore()
const proxyStore = useProxyStore()
const message = useMessage()

const showEditDialog = ref(false)
const showDeleteConfirm = ref(false)
const editingConfig = ref<ConfigMeta | null>(null)
const deletingConfigName = ref<string | null>(null)

onMounted(async () => {
  await configStore.fetchConfigs()
  await proxyStore.init()
})

const isActive = (name: string) => proxyStore.proxyState.active_config === name

function openCreateDialog() {
  editingConfig.value = null
  showEditDialog.value = true
}

function openEditDialog(config: ConfigMeta) {
  editingConfig.value = config
  showEditDialog.value = true
}

async function handleSaveConfig(data: { name: string; global_ip: string }) {
  try {
    if (editingConfig.value) {
      await configStore.saveConfig({
        name: editingConfig.value.name,
        global_ip: data.global_ip,
        ports: [],
      })
      message.success('配置已更新')
    } else {
      await configStore.saveConfig({
        name: data.name,
        global_ip: data.global_ip,
        ports: [],
      })
      message.success('配置已创建')
    }
  } catch (e: any) {
    message.error(`保存失败: ${e}`)
  }
}

async function handleActivate(name: string) {
  try {
    await proxyStore.switchConfig(name)
    await configStore.loadConfig(name)
    message.success(`已切换到配置: ${name}`)
  } catch (e: any) {
    message.error(`激活失败: ${e}`)
  }
}

function openDeleteConfirm(name: string) {
  deletingConfigName.value = name
  showDeleteConfirm.value = true
}

async function handleDeleteConfig() {
  if (!deletingConfigName.value) return
  try {
    await configStore.deleteConfig(deletingConfigName.value)
    message.success('配置已删除')
  } catch (e: any) {
    message.error(`删除失败: ${e}`)
  }
}
</script>

<template>
  <div>
    <div
      style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px"
    >
      <h2 style="margin: 0">配置管理</h2>
      <n-button type="primary" @click="openCreateDialog"> 新建配置 </n-button>
    </div>

    <n-card size="small" title="配置方案列表">
      <template v-if="configStore.configs.length === 0">
        <n-empty description="暂无配置方案" />
      </template>
      <n-list v-else>
        <n-list-item v-for="config in configStore.configs" :key="config.name">
          <n-space align="center" justify="space-between" style="width: 100%">
            <n-space align="center">
              <n-tag v-if="isActive(config.name)" type="success" size="small">
                当前激活
              </n-tag>
              <n-text strong>{{ config.name }}</n-text>
              <n-text depth="3">全局 IP: {{ config.global_ip }}</n-text>
              <n-text depth="3">
                端口: {{ config.enabled_count }}/{{ config.port_count }} 启用
              </n-text>
            </n-space>

            <n-space>
              <n-button
                v-if="!isActive(config.name)"
                size="small"
                type="primary"
                ghost
                @click="handleActivate(config.name)"
              >
                激活
              </n-button>
              <n-button size="small" @click="openEditDialog(config)"> 编辑 </n-button>
              <n-button
                size="small"
                type="error"
                :disabled="isActive(config.name)"
                @click="openDeleteConfirm(config.name)"
              >
                删除
              </n-button>
            </n-space>
          </n-space>
        </n-list-item>
      </n-list>
    </n-card>

    <ConfigEditDialog
      v-model:visible="showEditDialog"
      :edit-config="
        editingConfig
          ? { name: editingConfig.name, global_ip: editingConfig.global_ip }
          : null
      "
      @save="handleSaveConfig"
    />

    <ConfirmDialog
      v-model:visible="showDeleteConfirm"
      title="删除配置"
      :content="`确认删除配置「${deletingConfigName}」吗？此操作不可撤销。`"
      positive-text="删除"
      @confirm="handleDeleteConfig"
    />
  </div>
</template>
