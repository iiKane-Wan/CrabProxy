# CrabProxy 🦀

基于 **Rust + Tauri 2.x + Vue 3 + Naive UI** 构建的跨平台 TCP 端口代理工具。支持多配置文件管理、灵活的端口-IP-端口映射、一键开关代理、开机自启及明暗主题切换。

## 功能特性

- **多配置方案管理** — 创建、编辑、删除多个代理配置方案，每个方案独立存储
- **灵活端口映射** — 支持自定义监听端口 → 目标 IP → 目标端口的三元组映射
- **全局 / 单端口开关** — 一键开启/关闭所有代理，也可独立控制单个端口
- **配置热切换** — 切换配置方案时自动停止旧代理、启动新代理，端口正确释放
- **启动自动恢复** — 关闭应用后记住当前配置，下次启动自动恢复代理
- **实时状态监控** — 首页仪表盘展示每个端口的运行状态（运行中/已禁用/错误）
- **系统托盘** — 关闭窗口隐藏到托盘，双击恢复，右键菜单快速操作
- **开机自启** — 支持 Windows / macOS / Linux 系统启动时自动运行
- **主题切换** — 浅色 / 深色 / 跟随系统三种模式，窗口标题栏同步切换
- **跨平台** — 支持 Windows（MSI/NSIS）、macOS（DMG）、Linux（deb/AppImage/rpm）

## 页面预览

| 页面 | 功能 |
|------|------|
| **首页** | 配置方案选择下拉框、全局开关、端口状态列表（启用/禁用开关） |
| **代理管理** | 添加/编辑/删除端口规则，配置监听端口、目标 IP、目标端口 |
| **配置管理** | 新建/编辑/删除配置方案，一键激活，显示端口启用统计 |
| **设置** | 开机自启开关、主题选择（浅色/深色/跟随系统）、关于信息 |

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.x（Rust） |
| 前端 | Vue 3 (Composition API) + TypeScript + Vite |
| UI 组件 | Naive UI |
| 状态管理 | Pinia |
| 代理引擎 | Tokio 异步 TCP 转发 |
| 持久化 | JSON 配置文件（存储于用户配置目录） |

## 配置文件结构

配置存储在 `%APPDATA%/port-proxy/configs/`（Windows）或 `~/.config/port-proxy/configs/`（Linux/macOS）。

每个配置方案一个 JSON 文件：

```json
{
  "name": "工作环境",
  "global_ip": "192.168.1.100",
  "ports": [
    {
      "local_port": 8080,
      "target_ip": "10.0.0.1",
      "target_port": 9090,
      "enabled": true
    },
    {
      "local_port": 3306,
      "target_ip": null,
      "target_port": null,
      "enabled": false
    }
  ]
}
```

| 字段 | 必填 | 说明 |
|------|------|------|
| `name` | ✅ | 配置名称（唯一标识） |
| `global_ip` | ✅ | 全局默认目标 IP（IPv4） |
| `ports[].local_port` | ✅ | 本地监听端口（1–65535，同配置内唯一） |
| `ports[].target_ip` | ❌ | 目标 IP（留空或 null 则使用 global_ip） |
| `ports[].target_port` | ❌ | 目标端口（留空或 0 则等于 local_port） |
| `ports[].enabled` | ❌ | 是否启用（默认 true） |

## 开发环境要求

- **Node.js** ≥ 22
- **Rust** ≥ 1.77（stable）
- **系统依赖**：
  - Windows：无需额外依赖
  - Linux：`libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`
  - macOS：Xcode Command Line Tools

## 快速开始

```bash
# 克隆项目
git clone <repo-url> && cd crab-proxy

# 安装前端依赖
npm install

# 开发模式启动
npm run tauri dev

# 生产构建
npm run tauri build
```

## 项目结构

```
├── src/                        # Vue 3 前端
│   ├── main.ts                 # 应用入口
│   ├── App.vue                 # 根布局（导航栏 + 路由出口）
│   ├── router/index.ts         # 路由配置
│   ├── stores/                 # Pinia 状态管理
│   │   ├── configStore.ts      # 配置方案 CRUD
│   │   ├── proxyStore.ts       # 代理运行状态
│   │   └── settingsStore.ts    # 开机自启 + 主题
│   ├── views/                  # 页面组件
│   │   ├── DashboardView.vue   # 首页仪表盘
│   │   ├── ProxyManagementView.vue  # 代理管理
│   │   ├── ConfigManagementView.vue # 配置管理
│   │   └── SettingsView.vue    # 设置
│   ├── components/             # 可复用组件
│   │   ├── layout/AppSidebar.vue    # 左侧导航栏
│   │   ├── proxy/                   # 代理相关组件
│   │   ├── config/                  # 配置相关组件
│   │   └── common/                  # 通用组件
│   ├── types/index.ts          # TypeScript 类型定义
│   └── assets/                 # 静态资源
├── src-tauri/                  # Rust 后端
│   ├── Cargo.toml              # Rust 依赖
│   ├── tauri.conf.json         # Tauri 配置
│   ├── capabilities/           # 权限配置
│   ├── icons/                  # 应用图标
│   └── src/
│       ├── main.rs             # Rust 入口
│       ├── lib.rs              # 模块注册 + 托盘 + 命令注册
│       ├── config/             # 配置管理
│       │   ├── model.rs        # 数据模型（ProxyConfig, PortRule）
│       │   └── manager.rs      # JSON 文件读写
│       ├── proxy/              # 代理引擎
│       │   ├── engine.rs       # 引擎核心（任务编排）
│       │   ├── task.rs         # 单端口代理任务（TCP 转发）
│       │   └── state.rs        # 运行时状态
│       ├── commands/           # Tauri IPC 命令
│       │   ├── config_cmd.rs   # 配置 CRUD 命令
│       │   ├── proxy_cmd.rs    # 代理控制命令
│       │   └── system_cmd.rs   # 系统设置命令
│       ├── startup/mod.rs      # 开机自启
│       └── error.rs            # 统一错误类型
└── .github/workflows/          # CI/CD（三平台自动构建）
    └── build.yml
```

## 代理引擎架构

```
┌──────────────────────────────────┐
│         Vue 3 Frontend           │
│    invoke() ←──→ listen(event)   │
└──────────────┬───────────────────┘
               │ IPC
┌──────────────▼───────────────────┐
│       Tauri Commands (Rust)      │
│   switch_config / toggle / CRUD  │
└──────────────┬───────────────────┘
               │
┌──────────────▼───────────────────┐
│     ProxyEngine (Arc<Mutex<>>)   │
│   ┌──────────────────────────┐   │
│   │  PortTask (local_port)   │   │
│   │  ├─ TcpListener::accept  │   │
│   │  ├─ TcpStream::connect   │   │
│   │  └─ copy_bidirectional   │   │
│   └──────────────────────────┘   │
│   ┌──────────────────────────┐   │
│   │  PortTask (local_port)   │   │
│   │  ...                     │   │
│   └──────────────────────────┘   │
└──────────────────────────────────┘
```

每个端口对应一个独立的 Tokio 任务，通过 `Notify` 实现优雅关闭（发信号 → 停止 accept → 等待 5 秒 → 强制 abort），确保端口正确释放。

## 构建安装包

### Windows（本地直接构建）

```bash
npm run tauri build
# 产物：src-tauri/target/release/bundle/
#   nsis/CrabProxy_0.1.0_x64-setup.exe
#   msi/CrabProxy_0.1.0_x64_en-US.msi
```

### Linux / macOS（GitHub Actions 或本地构建）

项目包含 `.github/workflows/build.yml`，推送至 GitHub 后自动构建三平台安装包。也可在对应平台本地运行 `npm run tauri build`。

## License

MIT
