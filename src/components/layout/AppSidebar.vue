<script setup lang="ts">
import { h, type Component } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  HomeOutline,
  GitNetworkOutline,
  SettingsOutline,
  CogOutline,
} from '@vicons/ionicons5'

const router = useRouter()
const route = useRoute()

interface MenuItem {
  label: string
  key: string
  icon: Component
}

const menuItems: MenuItem[] = [
  { label: '首页', key: '/', icon: HomeOutline },
  { label: '代理', key: '/proxy', icon: GitNetworkOutline },
  { label: '配置', key: '/config', icon: SettingsOutline },
  { label: '设置', key: '/settings', icon: CogOutline },
]

const activeKey = computed(() => route.path)

function handleMenuClick(key: string) {
  router.push(key)
}

function renderIcon(icon: Component) {
  return () => h(icon)
}
</script>

<template>
  <n-layout-sider
    bordered
    collapse-mode="width"
    :collapsed-width="64"
    :width="220"
    :native-scrollbar="false"
    style="background: var(--n-color-embedded)"
  >
    <!-- Logo 区域 -->
    <div
      style="
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 20px 18px;
        border-bottom: 1px solid var(--n-border-color);
      "
    >
      <img
        src="../../assets/logo.png"
        alt="CrabProxy"
        style="width: 28px; height: 28px; border-radius: 6px"
      />
      <span
        style="
          font-size: 16px;
          font-weight: 600;
          white-space: nowrap;
          color: var(--n-text-color);
        "
      >
        CrabProxy
      </span>
    </div>

    <!-- 菜单 -->
    <n-menu
      :value="activeKey"
      :options="
        menuItems.map((item) => ({
          label: item.label,
          key: item.key,
          icon: renderIcon(item.icon),
        }))
      "
      style="margin-top: 8px"
      @update:value="handleMenuClick"
    />

    <!-- 底部版本 -->
    <div
      style="
        position: absolute;
        bottom: 16px;
        left: 0;
        right: 0;
        text-align: center;
        font-size: 12px;
        color: var(--n-text-color-disabled);
      "
    >
      CrabProxy v0.1.0
    </div>
  </n-layout-sider>
</template>
