import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('../views/DashboardView.vue'),
    },
    {
      path: '/proxy',
      name: 'proxy',
      component: () => import('../views/ProxyManagementView.vue'),
    },
    {
      path: '/config',
      name: 'config',
      component: () => import('../views/ConfigManagementView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsView.vue'),
    },
  ],
})

export default router
