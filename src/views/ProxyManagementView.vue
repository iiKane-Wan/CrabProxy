<script setup lang="ts">
import { useConfigStore } from '../stores/configStore'
import { useProxyStore } from '../stores/proxyStore'
import ProxyStatusBar from '../components/proxy/ProxyStatusBar.vue'
import PortEditDialog from '../components/proxy/PortEditDialog.vue'
import ConfirmDialog from '../components/common/ConfirmDialog.vue'
import { useMessage } from 'naive-ui'
import type { PortRule } from '../types'

const configStore = useConfigStore()
const proxyStore = useProxyStore()
const message = useMessage()

const showAddDialog = ref(false)
const showDeleteConfirm = ref(false)
const editingPort = ref<PortRule | null>(null)
const deletingPort = ref<{ local_port: number } | null>(null)

onMounted(async () => {
  await configStore.fetchConfigs()
  await proxyStore.init()
})

const globalIp = computed(() => configStore.currentConfig?.global_ip || '')

// 全局开关
async function handleGlobalToggle(enabled: boolean) {
  if (!configStore.currentConfig) {
    message.warning('请先选择配置')
    return
  }
  try {
    await proxyStore.toggleAllPorts(configStore.currentConfig.name, enabled)
  } catch (e: any) {
    message.error(`操作失败: ${e}`)
  }
}

// 添加端口
async function handleAddPort(port: PortRule) {
  if (!configStore.currentConfig) return
  try {
    await proxyStore.addPort(configStore.currentConfig.name, port)
    await configStore.loadConfig(configStore.currentConfig.name)
    message.success('端口已添加')
  } catch (e: any) {
    message.error(`添加失败: ${e}`)
  }
}

// 编辑端口
function openEditDialog(port: { local_port: number; target_ip: string; target_port: number | null; enabled: boolean }) {
  editingPort.value = {
    local_port: port.local_port,
    target_ip: port.target_ip || null,
    target_port: port.target_port || null,
    enabled: port.enabled,
  }
  showAddDialog.value = true
}

async function handleEditPort(port: PortRule) {
  if (!configStore.currentConfig) return
  try {
    await proxyStore.updatePort(configStore.currentConfig.name, port)
    await configStore.loadConfig(configStore.currentConfig.name)
    message.success('端口已更新')
  } catch (e: any) {
    message.error(`更新失败: ${e}`)
  }
}

// 删除端口
function openDeleteConfirm(port: { local_port: number }) {
  deletingPort.value = port
  showDeleteConfirm.value = true
}

async function handleDeletePort() {
  if (!configStore.currentConfig || !deletingPort.value) return
  try {
    await proxyStore.removePort(configStore.currentConfig.name, deletingPort.value.local_port)
    await configStore.loadConfig(configStore.currentConfig.name)
    message.success('端口已删除')
  } catch (e: any) {
    message.error(`删除失败: ${e}`)
  }
}

// 单端口开关切换
async function handlePortToggle(port: { local_port: number }, enabled: boolean) {
  if (!configStore.currentConfig) return
  const rule = configStore.currentConfig.ports.find(
    (p) => p.local_port === port.local_port,
  )
  if (!rule) return
  try {
    await proxyStore.updatePort(configStore.currentConfig.name, {
      local_port: rule.local_port,
      target_ip: rule.target_ip || null,
      target_port: rule.target_port || null,
      enabled,
    })
    await configStore.loadConfig(configStore.currentConfig.name)
  } catch (e: any) {
    message.error(`操作失败: ${e}`)
  }
}
</script>

<template>
  <div>
    <div
      style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px"
    >
      <h2 style="margin: 0">代理管理</h2>
      <n-button
        type="primary"
        :disabled="!configStore.currentConfig"
        @click="editingPort = null; showAddDialog = true"
      >
        添加端口
      </n-button>
    </div>

    <ProxyStatusBar
      :global-enabled="proxyStore.proxyState.global_enabled"
      :loading="proxyStore.isLoading"
      :active-config-name="proxyStore.proxyState.active_config"
      @update:global-enabled="handleGlobalToggle"
    />

    <!-- 端口列表 -->
    <n-card size="small" title="端口列表">
      <template v-if="configStore.currentConfig === null">
        <n-empty description="请先在首页或配置页选择并激活一个配置方案" />
      </template>
      <template v-else-if="configStore.currentConfig.ports.length === 0">
        <n-empty description="暂无端口，点击上方按钮添加" />
      </template>
      <n-list v-else>
        <n-list-item
          v-for="port in configStore.currentConfig.ports"
          :key="port.local_port"
        >
          <n-space align="center" justify="space-between" style="width: 100%">
            <n-space align="center">
              <n-tag type="info" size="small">{{ port.local_port }}</n-tag>
              <n-text
                >→ {{ port.target_ip || globalIp }}:{{ port.target_port || port.local_port }}</n-text
              >
            </n-space>

            <n-space align="center">
              <n-switch
                :value="port.enabled"
                size="small"
                @update:value="(v: boolean) => handlePortToggle(port, v)"
              />
              <n-button size="tiny" @click="openEditDialog({local_port: port.local_port, target_ip: port.target_ip || '', target_port: port.target_port || null, enabled: port.enabled})"
                >编辑</n-button
              >
              <n-button size="tiny" type="error" @click="openDeleteConfirm(port)">
                删除
              </n-button>
            </n-space>
          </n-space>
        </n-list-item>
      </n-list>
    </n-card>

    <!-- 弹窗 -->
    <PortEditDialog
      v-model:visible="showAddDialog"
      :global-ip="globalIp"
      :edit-port="editingPort"
      @save="editingPort ? handleEditPort($event) : handleAddPort($event)"
    />

    <ConfirmDialog
      v-model:visible="showDeleteConfirm"
      title="删除端口"
      :content="`确认删除端口 ${deletingPort?.local_port} 吗？此操作不可撤销。`"
      positive-text="删除"
      @confirm="handleDeletePort"
    />
  </div>
</template>
