use serde::Serialize;
use std::fmt;

/// 应用程序统一错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 解析错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("配置错误: {0}")]
    Config(String),

    #[error("代理错误: {0}")]
    Proxy(String),

    #[error("系统错误: {0}")]
    System(String),
}

/// 类型别名：简化 Result 使用
pub type AppResult<T> = Result<T, AppError>;

/// 用于 Tauri 命令返回值的可序列化错误
#[derive(Debug, Clone, Serialize)]
pub struct CommandError {
    pub message: String,
}

impl From<AppError> for CommandError {
    fn from(err: AppError) -> Self {
        CommandError {
            message: err.to_string(),
        }
    }
}

impl From<String> for CommandError {
    fn from(s: String) -> Self {
        CommandError { message: s }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
