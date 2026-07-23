use crate::config::model::ProxyConfig;
use crate::proxy::state::{PortState, ProxyState};
use crate::proxy::task::PortTask;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

/// 代理引擎：管理所有端口代理任务
pub struct ProxyEngine {
    active_config: Option<String>,
    global_enabled: bool,
    tasks: HashMap<u16, PortTask>,
    port_states: Vec<PortState>,
}

impl ProxyEngine {
    pub fn new() -> Self {
        ProxyEngine {
            active_config: None,
            global_enabled: false,
            tasks: HashMap::new(),
            port_states: Vec::new(),
        }
    }

    /// 加载并启动配置
    pub async fn load_config(
        &mut self,
        config: &ProxyConfig,
        app_handle: &tauri::AppHandle,
    ) -> Result<(), String> {
        self.stop_all().await;

        self.active_config = Some(config.name.clone());
        self.global_enabled = true;
        self.port_states = config
            .ports
            .iter()
            .map(|p| PortState {
                local_port: p.local_port,
                target_ip: config.resolve_target_ip(p),
                target_port: config.resolve_target_port(p),
                enabled: p.enabled,
                running: false,
                error: None,
            })
            .collect();

        // 收集需要启动的端口（避免同时持有 self 的多个可变引用）
        let to_start: Vec<(u16, String, u16)> = self
            .port_states
            .iter()
            .filter(|p| p.enabled)
            .map(|p| (p.local_port, p.target_ip.clone(), p.target_port))
            .collect();

        let app_clone = app_handle.clone();
        for (local_port, target_ip, target_port) in to_start {
            self.start_port_task(local_port, target_ip, target_port, &app_clone).await;
        }

        let _ = app_handle.emit("proxy-state-changed", self.snapshot());
        log::info!("配置 '{}' 已加载，共 {} 个端口", config.name, config.ports.len());
        Ok(())
    }

    /// 启动单个端口的代理任务
    async fn start_port_task(
        &mut self,
        local_port: u16,
        target_ip: String,
        target_port: u16,
        app_handle: &tauri::AppHandle,
    ) {
        if self.tasks.contains_key(&local_port) {
            if let Some(ps) = self.port_states.iter_mut().find(|p| p.local_port == local_port) {
                ps.running = true;
            }
            return;
        }

        let app_clone = app_handle.clone();
        match PortTask::spawn(local_port, target_ip, target_port, move |err| {
            let _ = app_clone.emit("proxy-error", err);
        })
        .await
        {
            Ok(task) => {
                self.tasks.insert(local_port, task);
                if let Some(ps) = self.port_states.iter_mut().find(|p| p.local_port == local_port) {
                    ps.running = true;
                    ps.error = None;
                }
            }
            Err(e) => {
                if let Some(ps) = self.port_states.iter_mut().find(|p| p.local_port == local_port) {
                    ps.running = false;
                    ps.error = Some(e.clone());
                }
                log::error!("启动端口 {} 代理失败: {}", local_port, e);
                let _ = app_handle.emit("proxy-error", e);
            }
        }
    }

    /// 停止单个端口的代理任务（等待端口真正释放）
    async fn stop_port_task(&mut self, local_port: u16) {
        if let Some(task) = self.tasks.remove(&local_port) {
            // 等待 accept 循环退出并释放端口（最多 5 秒超时）
            task.shutdown().await;
        }
        if let Some(ps) = self.port_states.iter_mut().find(|p| p.local_port == local_port) {
            ps.running = false;
            ps.error = None;
        }
    }

    /// 停止所有代理任务（等待所有端口释放）
    pub async fn stop_all(&mut self) {
        let ports: Vec<u16> = self.tasks.keys().copied().collect();
        for port in ports {
            self.stop_port_task(port).await;
        }
        self.global_enabled = false;
    }

    /// 全局批量切换
    pub async fn toggle_all(
        &mut self,
        enabled: bool,
        config: Option<&ProxyConfig>,
        app_handle: &tauri::AppHandle,
    ) -> Result<(), String> {
        if enabled {
            let _config = config.ok_or("没有激活的配置")?;
            self.global_enabled = true;

            let to_start: Vec<(u16, String, u16)> = self
                .port_states
                .iter()
                .filter(|p| p.enabled)
                .map(|p| (p.local_port, p.target_ip.clone(), p.target_port))
                .collect();

            let app_clone = app_handle.clone();
            for (local_port, target_ip, target_port) in to_start {
                self.start_port_task(local_port, target_ip, target_port, &app_clone).await;
            }
        } else {
            self.stop_all().await;
        }

        let _ = app_handle.emit("proxy-state-changed", self.snapshot());
        Ok(())
    }

    /// 动态添加端口
    pub async fn add_port(
        &mut self,
        local_port: u16,
        target_ip: String,
        target_port: u16,
        enabled: bool,
        app_handle: &tauri::AppHandle,
    ) -> Result<(), String> {
        if self.port_states.iter().any(|p| p.local_port == local_port) {
            return Err(format!("端口 {} 已存在", local_port));
        }

        let running = enabled && self.global_enabled;

        self.port_states.push(PortState {
            local_port,
            target_ip: target_ip.clone(),
            target_port,
            enabled,
            running: false,
            error: None,
        });

        if running {
            self.start_port_task(local_port, target_ip, target_port, app_handle).await;
        }

        let _ = app_handle.emit("proxy-state-changed", self.snapshot());
        Ok(())
    }

    /// 动态移除端口
    pub async fn remove_port(
        &mut self,
        local_port: u16,
        app_handle: &tauri::AppHandle,
    ) -> Result<(), String> {
        self.stop_port_task(local_port).await;
        self.port_states.retain(|p| p.local_port != local_port);
        let _ = app_handle.emit("proxy-state-changed", self.snapshot());
        Ok(())
    }

    /// 更新端口配置
    pub async fn update_port(
        &mut self,
        local_port: u16,
        target_ip: String,
        target_port: u16,
        enabled: bool,
        app_handle: &tauri::AppHandle,
    ) -> Result<(), String> {
        let (old_target_ip, old_target_port, old_enabled) = {
            let ps = self
                .port_states
                .iter()
                .find(|p| p.local_port == local_port)
                .ok_or_else(|| format!("端口 {} 不存在", local_port))?;
            (ps.target_ip.clone(), ps.target_port, ps.enabled)
        };

        let needs_restart =
            old_target_ip != target_ip || old_target_port != target_port || old_enabled != enabled;

        if let Some(ps) = self.port_states.iter_mut().find(|p| p.local_port == local_port) {
            ps.target_ip = target_ip.clone();
            ps.target_port = target_port;
            ps.enabled = enabled;
        }

        if needs_restart {
            self.stop_port_task(local_port).await;
            if enabled && self.global_enabled {
                self.start_port_task(local_port, target_ip, target_port, app_handle).await;
            }
        }

        let _ = app_handle.emit("proxy-state-changed", self.snapshot());
        Ok(())
    }

    /// 获取当前状态快照
    pub fn snapshot(&self) -> ProxyState {
        ProxyState {
            active_config: self.active_config.clone(),
            global_enabled: self.global_enabled,
            ports: self.port_states.clone(),
        }
    }
}

/// 线程安全的代理引擎包装类型
pub type SharedProxyEngine = Arc<Mutex<ProxyEngine>>;

/// 创建共享代理引擎实例
pub fn create_shared_engine() -> SharedProxyEngine {
    Arc::new(Mutex::new(ProxyEngine::new()))
}
