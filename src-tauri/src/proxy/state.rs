use serde::{Deserialize, Serialize};

/// 单个端口的运行时状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortState {
    pub local_port: u16,
    /// 端口名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub target_ip: String,
    pub target_port: u16,
    pub enabled: bool,
    pub running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 代理引擎全局状态快照（发送给前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyState {
    /// 当前激活的配置名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_config: Option<String>,
    /// 全局开关
    pub global_enabled: bool,
    /// 各端口运行状态
    pub ports: Vec<PortState>,
}

impl Default for ProxyState {
    fn default() -> Self {
        ProxyState {
            active_config: None,
            global_enabled: false,
            ports: Vec::new(),
        }
    }
}
