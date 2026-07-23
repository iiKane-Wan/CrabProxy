<script setup lang="ts">
import { useConfigStore } from '../stores/configStore'
import { useProxyStore } from '../stores/proxyStore'
import ProxyStatusBar from '../components/proxy/ProxyStatusBar.vue'
import { useMessage } from 'naive-ui'

const configStore = useConfigStore()
const proxyStore = useProxyStore()
const message = useMessage()

const selectedConfig = ref<string | null>(null)
const configOptions = computed(() =>
  configStore.configs.map((c) => ({
    label: c.name,
    value: c.name,
  })),
)

onMounted(async () => {
  await configStore.fetchConfigs()

  // 尝试恢复上次会话的配置（内部会启动代理引擎）
  const restoredName = await proxyStore.restoreLastSession()

  // 初始化事件监听
  await proxyStore.init()

  // 同步下拉框：优先用刚恢复的配置名，其次取引擎当前激活的
  const activeName = restoredName || proxyStore.proxyState.active_config
  if (activeName) {
    selectedConfig.value = activeName
    try {
      await configStore.loadConfig(activeName)
    } catch {
      // 配置可能已被删除，忽略
    }
  }
})

async function handleConfigSwitch(name: string) {
  try {
    await proxyStore.switchConfig(name)
    await configStore.loadConfig(name)
    selectedConfig.value = name
  } catch (e: any) {
    message.error(`切换配置失败: ${e}`)
  }
}

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

async function handlePortToggle(port: { local_port: number }, enabled: boolean) {
  if (!configStore.currentConfig) return
  const cfg = configStore.currentConfig
  const rule = cfg.ports.find((p) => p.local_port === port.local_port)
  if (!rule) return
  try {
    await proxyStore.updatePort(cfg.name, {
      ...rule,
      enabled,
    })
  } catch (e: any) {
    message.error(`操作失败: ${e}`)
  }
}
</script>

<template>
  <div>
    <h2 style="margin-top: 0; margin-bottom: 16px">首页</h2>

    <!-- 配置选择 -->
    <n-card size="small" style="margin-bottom: 12px">
      <n-space align="center">
        <span style="font-weight: 500">当前配置：</span>
        <n-select
          v-model:value="selectedConfig"
          :options="configOptions"
          placeholder="选择配置方案"
          style="width: 240px"
          @update:value="handleConfigSwitch"
        />
        <n-button size="small" @click="$router.push('/config')"> 管理配置 </n-button>
      </n-space>
    </n-card>

    <!-- 全局开关 -->
    <ProxyStatusBar
      :global-enabled="proxyStore.proxyState.global_enabled"
      :loading="proxyStore.isLoading"
      :active-config-name="proxyStore.proxyState.active_config"
      @update:global-enabled="handleGlobalToggle"
    />

    <!-- 端口列表 -->
    <n-card size="small" title="端口状态">
      <template v-if="proxyStore.proxyState.ports.length === 0">
        <n-empty description="当前配置下无端口，请前往代理页添加" />
      </template>
      <n-list v-else>
        <n-list-item
          v-for="port in proxyStore.proxyState.ports"
          :key="port.local_port"
        >
          <template #prefix>
            <n-tag
              :type="port.running ? 'success' : port.enabled ? 'warning' : 'default'"
              size="small"
            >
              {{ port.running ? '运行中' : port.enabled ? '待启动' : '已禁用' }}
            </n-tag>
          </template>

          <n-space align="center" justify="space-between">
            <div>
              <n-text strong>端口 {{ port.local_port }}</n-text>
              <n-text depth="3" style="margin-left: 12px">
                → {{ port.target_ip }}:{{ port.target_port }}
              </n-text>
              <n-text
                v-if="port.error"
                type="error"
                style="margin-left: 8px; font-size: 12px"
              >
                {{ port.error }}
              </n-text>
            </div>
            <n-switch
              :value="port.enabled"
              :loading="proxyStore.isLoading"
              @update:value="(v: boolean) => handlePortToggle(port, v)"
            />
          </n-space>
        </n-list-item>
      </n-list>
    </n-card>
  </div>
</template>
