use std::sync::Arc;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tokio::task::{AbortHandle, JoinHandle};

/// 单个端口代理任务
pub struct PortTask {
    /// 本地监听端口
    pub local_port: u16,
    /// 关闭通知信号（用于优雅停止 accept 循环）
    shutdown_notify: Arc<Notify>,
    /// 任务句柄（用于等待 accept 循环退出）
    join_handle: JoinHandle<()>,
    /// 强制中止句柄（超时后备方案）
    abort_handle: AbortHandle,
}

impl PortTask {
    /// 启动端口代理任务
    pub async fn spawn(
        local_port: u16,
        target_ip: String,
        target_port: u16,
        on_error: impl Fn(String) + Send + 'static,
    ) -> Result<Self, String> {
        let bind_addr = format!("0.0.0.0:{}", local_port);
        let listener = TcpListener::bind(&bind_addr)
            .await
            .map_err(|e| format!("端口 {} 监听失败: {}", local_port, e))?;

        let shutdown_notify = Arc::new(Notify::new());
        let shutdown_clone = shutdown_notify.clone();
        let target = target_ip.clone();

        let join_handle = tokio::spawn(async move {
            run_accept_loop(
                listener,
                target,
                local_port,
                target_port,
                shutdown_clone,
                on_error,
            )
            .await;
        });

        log::info!(
            "端口 {} 代理已启动 -> {}:{}",
            local_port,
            target_ip,
            target_port
        );

        Ok(PortTask {
            local_port,
            shutdown_notify,
            abort_handle: join_handle.abort_handle(),
            join_handle,
        })
    }

    /// 优雅关闭：发送停止信号 → 等待 accept 循环退出 → 超时则强制中止
    pub async fn shutdown(self) {
        log::info!("正在停止端口 {} 代理...", self.local_port);

        // 1. 发送关闭信号，停止接受新连接
        self.shutdown_notify.notify_one();

        // 2. 等待 accept 循环退出（最多 5 秒）
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.join_handle,
        )
        .await
        {
            Ok(_) => {
                log::info!("端口 {} 代理已正常停止", self.local_port);
            }
            Err(_) => {
                // 超时，强制中止
                log::warn!("端口 {} 代理停止超时，强制中止", self.local_port);
                self.abort_handle.abort();
            }
        }
    }
}

/// accept 循环：监听新连接并为每个连接创建处理任务
async fn run_accept_loop(
    listener: TcpListener,
    target_ip: String,
    local_port: u16,
    target_port: u16,
    shutdown: Arc<Notify>,
    on_error: impl Fn(String) + Send + 'static,
) {
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((client_stream, client_addr)) => {
                        let target = target_ip.clone();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(
                                client_stream,
                                &target,
                                local_port,
                                target_port,
                            ).await {
                                log::debug!("端口 {} 连接处理错误 ({}): {}", local_port, client_addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        log::error!("端口 {} accept 错误: {}", local_port, e);
                        on_error(format!("端口 {} 监听异常: {}", local_port, e));
                        break;
                    }
                }
            }
            _ = shutdown.notified() => {
                log::info!("端口 {} 收到关闭信号，停止接受新连接", local_port);
                break;
            }
        }
    }

    // accept 循环退出，drop listener 释放端口
    drop(listener);
    log::info!("端口 {} 监听已关闭，端口已释放", local_port);
}

/// 处理单条客户端连接：连接到目标 → 双向数据转发
async fn handle_connection(
    client_stream: TcpStream,
    target_ip: &str,
    local_port: u16,
    target_port: u16,
) -> io::Result<()> {
    let target_addr = format!("{}:{}", target_ip, target_port);

    let target_stream = match tokio::time::timeout(
        std::time::Duration::from_secs(10),
        TcpStream::connect(&target_addr),
    )
    .await
    {
        Ok(Ok(stream)) => stream,
        Ok(Err(e)) => {
            log::warn!("端口 {} 连接目标 {} 失败: {}", local_port, target_addr, e);
            return Err(e);
        }
        Err(_) => {
            log::warn!("端口 {} 连接目标 {} 超时", local_port, target_addr);
            return Err(io::Error::new(io::ErrorKind::TimedOut, "连接目标超时"));
        }
    };

    log::debug!("端口 {} 已建立转发: -> {}", local_port, target_addr);

    let (mut cr, mut cw) = client_stream.into_split();
    let (mut tr, mut tw) = target_stream.into_split();

    let client_to_target = io::copy(&mut cr, &mut tw);
    let target_to_client = io::copy(&mut tr, &mut cw);

    tokio::select! {
        res = client_to_target => {
            if let Err(e) = res {
                log::debug!("端口 {} 客户端→目标 转发错误: {}", local_port, e);
            }
        }
        res = target_to_client => {
            if let Err(e) = res {
                log::debug!("端口 {} 目标→客户端 转发错误: {}", local_port, e);
            }
        }
    }

    log::debug!("端口 {} 连接已关闭", local_port);
    Ok(())
}
