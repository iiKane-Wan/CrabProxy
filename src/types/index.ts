// 端口代理工具 — TypeScript 类型定义

/** 端口规则 */
export interface PortRule {
  local_port: number
  target_ip: string | null // null 表示使用全局 IP
  target_port: number | null // null 或 0 表示使用本地端口
  enabled: boolean
}

/** 代理配置 */
export interface ProxyConfig {
  name: string
  global_ip: string
  ports: PortRule[]
}

/** 配置摘要（列表展示用） */
export interface ConfigMeta {
  name: string
  global_ip: string
  port_count: number
  enabled_count: number
}

/** 端口运行时状态 */
export interface PortState {
  local_port: number
  target_ip: string
  target_port: number
  enabled: boolean
  running: boolean
  error: string | null
}

/** 代理全局状态 */
export interface ProxyState {
  active_config: string | null
  global_enabled: boolean
  ports: PortState[]
}

/** 主题类型 */
export type ThemeMode = 'light' | 'dark' | 'system'
