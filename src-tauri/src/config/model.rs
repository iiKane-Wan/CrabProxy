use serde::{Deserialize, Serialize};

/// 端口代理规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRule {
    /// 端口名称（可选，仅用于标识）
    #[serde(default)]
    pub name: Option<String>,
    /// 本地监听端口
    pub local_port: u16,
    /// 目标 IP（None 或空字符串时使用配置的 global_ip）
    pub target_ip: Option<String>,
    /// 目标端口（None 或 0 时默认等于 local_port）
    #[serde(default)]
    pub target_port: Option<u16>,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// 代理配置方案
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    /// 配置名称（唯一标识，也用作文件名）
    pub name: String,
    /// 全局默认目标 IP
    pub global_ip: String,
    /// 端口规则列表
    #[serde(default)]
    pub ports: Vec<PortRule>,
}

/// 配置摘要信息（列表展示用，不包含端口详情）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMeta {
    pub name: String,
    pub global_ip: String,
    pub port_count: usize,
    pub enabled_count: usize,
}

impl ProxyConfig {
    /// 校验配置有效性
    pub fn validate(&self) -> Result<(), String> {
        // 校验名称
        if self.name.trim().is_empty() {
            return Err("配置名称不能为空".into());
        }
        if self.name.contains(|c: char| c == '/' || c == '\\' || c == ':') {
            return Err("配置名称不能包含 / \\ : 等特殊字符".into());
        }

        // 校验全局 IP
        let trimmed_ip = self.global_ip.trim();
        if trimmed_ip.is_empty() {
            return Err("全局默认 IP 不能为空".into());
        }
        if trimmed_ip.parse::<std::net::Ipv4Addr>().is_err() {
            return Err(format!("全局 IP '{}' 不是有效的 IPv4 地址", trimmed_ip));
        }

        // 校验端口规则
        let mut seen_ports = std::collections::HashSet::new();
        for port in &self.ports {
            // 检查端口范围
            if port.local_port == 0 {
                return Err("端口号不能为 0".into());
            }

            // 检查端口唯一性
            if !seen_ports.insert(port.local_port) {
                return Err(format!("端口 {} 重复", port.local_port));
            }

            // 校验目标 IP 格式（如果提供）
            if let Some(ref ip) = port.target_ip {
                let ip = ip.trim();
                if !ip.is_empty() && ip.parse::<std::net::Ipv4Addr>().is_err() {
                    return Err(format!("端口 {} 的目标 IP '{}' 不是有效的 IPv4 地址", port.local_port, ip));
                }
            }

            // 校验目标端口（如果提供，不可为 0；u16 类型保证 ≤65535）
            if let Some(tp) = port.target_port {
                if tp == 0 {
                    return Err(format!("端口 {} 的目标端口不可为 0", port.local_port));
                }
            }
        }

        Ok(())
    }

    /// 获取某个端口规则的实际目标 IP（若端口未指定则使用全局 IP）
    pub fn resolve_target_ip(&self, port: &PortRule) -> String {
        match &port.target_ip {
            Some(ip) if !ip.trim().is_empty() => ip.trim().to_string(),
            _ => self.global_ip.trim().to_string(),
        }
    }

    /// 获取某个端口规则的实际目标端口（若未指定则使用本地端口）
    pub fn resolve_target_port(&self, port: &PortRule) -> u16 {
        port.target_port.filter(|&p| p > 0).unwrap_or(port.local_port)
    }

    /// 生成配置摘要
    pub fn to_meta(&self) -> ConfigMeta {
        ConfigMeta {
            name: self.name.clone(),
            global_ip: self.global_ip.clone(),
            port_count: self.ports.len(),
            enabled_count: self.ports.iter().filter(|p| p.enabled).count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_config() {
        let config = ProxyConfig {
            name: "测试".into(),
            global_ip: "192.168.1.1".into(),
            ports: vec![
                PortRule {
                    name: None,
                    local_port: 8080,
                    target_ip: None,
                    target_port: None,
                    enabled: true,
                },
                PortRule {
                    name: None,
                    local_port: 3306,
                    target_ip: Some("10.0.0.1".into()),
                    target_port: Some(3307),
                    enabled: false,
                },
            ],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_duplicate_port() {
        let config = ProxyConfig {
            name: "测试".into(),
            global_ip: "192.168.1.1".into(),
            ports: vec![
                PortRule { name: None, local_port: 8080, target_ip: None, target_port: None, enabled: true },
                PortRule { name: None, local_port: 8080, target_ip: None, target_port: None, enabled: true },
            ],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_empty_name() {
        let config = ProxyConfig {
            name: "".into(),
            global_ip: "192.168.1.1".into(),
            ports: vec![],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_ip() {
        let config = ProxyConfig {
            name: "测试".into(),
            global_ip: "not-an-ip".into(),
            ports: vec![],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_target_port() {
        let config = ProxyConfig {
            name: "测试".into(),
            global_ip: "192.168.1.1".into(),
            ports: vec![
                PortRule { name: None, local_port: 8080, target_ip: None, target_port: Some(0), enabled: true },
            ],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_resolve_target_ip() {
        let config = ProxyConfig {
            name: "测试".into(),
            global_ip: "192.168.1.1".into(),
            ports: vec![],
        };

        let port1 = PortRule { name: None, local_port: 80, target_ip: None, target_port: None, enabled: true };
        assert_eq!(config.resolve_target_ip(&port1), "192.168.1.1");

        let port2 = PortRule { name: None, local_port: 443, target_ip: Some("10.0.0.5".into()), target_port: None, enabled: true };
        assert_eq!(config.resolve_target_ip(&port2), "10.0.0.5");

        let port3 = PortRule { name: None, local_port: 22, target_ip: Some("".into()), target_port: None, enabled: true };
        assert_eq!(config.resolve_target_ip(&port3), "192.168.1.1");
    }

    #[test]
    fn test_resolve_target_port() {
        let config = ProxyConfig {
            name: "测试".into(),
            global_ip: "192.168.1.1".into(),
            ports: vec![],
        };

        // 未指定 → 回退到 local_port
        let p1 = PortRule { name: None, local_port: 8080, target_ip: None, target_port: None, enabled: true };
        assert_eq!(config.resolve_target_port(&p1), 8080);

        // 指定 0 → 回退到 local_port
        let p2 = PortRule { name: None, local_port: 8080, target_ip: None, target_port: Some(0), enabled: true };
        assert_eq!(config.resolve_target_port(&p2), 8080);

        // 指定 9090 → 使用指定值
        let p3 = PortRule { name: None, local_port: 8080, target_ip: None, target_port: Some(9090), enabled: true };
        assert_eq!(config.resolve_target_port(&p3), 9090);
    }

    #[test]
    fn test_deserialize_old_config() {
        // 模拟旧配置文件（没有 target_port 字段），确保向后兼容
        let json = r#"{
            "name": "旧配置",
            "global_ip": "192.168.1.1",
            "ports": [
                {"local_port": 8080, "target_ip": null, "enabled": true}
            ]
        }"#;
        let config: ProxyConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.ports[0].target_port, None);
        assert_eq!(config.resolve_target_port(&config.ports[0]), 8080);
    }
}
