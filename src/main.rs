use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use clap::{Arg, Command};

mod config;
mod daemon;
mod health_check;
mod load_balancer;
mod interface_monitor;
mod udp_race;
mod mptcp;
mod nftables;

use config::Config;
use daemon::{DaemonManager, setup_signal_handlers};
use health_check::HealthChecker;
use load_balancer::LoadBalancer;
use interface_monitor::InterfaceMonitor;
use udp_race::UdpRaceManager;
use mptcp::MptcpManager;
use nftables::NftablesManager;

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let matches = Command::new("mwan3-nft")
        .version("1.0.0")
        .about("多WAN负载均衡工具")
        .arg(Arg::new("config")
            .short('c')
            .long("config")
            .value_name("FILE")
            .help("配置文件路径")
            .default_value("mwan3-nft.yaml"))
        .arg(Arg::new("daemon")
            .short('d')
            .long("daemon")
            .help("以daemon模式运行")
            .action(clap::ArgAction::SetTrue))
        .arg(Arg::new("pid-file")
            .short('p')
            .long("pid-file")
            .value_name("FILE")
            .help("PID文件路径")
            .default_value("/var/run/mwan3-nft.pid"))
        .arg(Arg::new("stop")
            .long("stop")
            .help("停止daemon进程")
            .action(clap::ArgAction::SetTrue))
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let pid_file = matches.get_one::<String>("pid-file").unwrap();
    let daemon_mode = matches.get_flag("daemon");
    let stop_daemon = matches.get_flag("stop");

    // 初始化日志
    tracing_subscriber::fmt::init();

    // Daemon管理器
    let daemon_manager = DaemonManager::new(pid_file.clone());

    // 处理停止daemon命令
    if stop_daemon {
        daemon_manager.stop_daemon()?;
        return Ok(());
    }

    // 检查是否已经在运行
    if daemon_manager.is_running() {
        tracing::error!("mwan3-nft 已经在运行中");
        return Ok(());
    }

    // 如果是daemon模式，进行daemon化
    if daemon_mode {
        daemon_manager.daemonize()?;
    }

    // 设置信号处理器
    setup_signal_handlers()?;

    // 加载配置
    let config = Arc::new(RwLock::new(Config::load(config_path).await?));

    // 配置已加载
    tracing::info!("配置文件已加载: {}", config_path);

    // 初始化各个管理器
    let nftables_manager = Arc::new(NftablesManager::new());
    let health_checker = Arc::new(HealthChecker::new(config.clone()));
    let load_balancer = Arc::new(LoadBalancer::new(config.clone(), health_checker.clone()));
    let interface_monitor = Arc::new(InterfaceMonitor::new(config.clone(), load_balancer.clone()));
    let udp_race_manager = Arc::new(UdpRaceManager::new(config.clone()));
    let mptcp_manager = Arc::new(MptcpManager::new(config.clone()));

    // 启动所有服务
    tracing::info!("启动 mwan3-nft 服务...");

    // 启动各个管理器的异步任务占位
    let health_handle = tokio::spawn(async move {
        if let Err(e) = health_checker.start().await {
            tracing::error!("健康检测器错误: {}", e);
        }
    });

    let interface_handle = tokio::spawn(async move {
        if let Err(e) = interface_monitor.start().await {
            tracing::error!("接口监控器错误: {}", e);
        }
    });

    let load_balancer_handle = tokio::spawn(async move {
        if let Err(e) = load_balancer.start().await {
            tracing::error!("负载均衡器错误: {}", e);
        }
    });

    let udp_race_handle = tokio::spawn(async move {
        if let Err(e) = udp_race_manager.start().await {
            tracing::error!("UDP Race管理器错误: {}", e);
        }
    });

    let mptcp_handle = tokio::spawn(async move {
        if let Err(e) = mptcp_manager.start().await {
            tracing::error!("MPTCP管理器错误: {}", e);
        }
    });

    // 保持程序运行
    tokio::signal::ctrl_c().await?;
    tracing::info!("收到停止信号，正在关闭...");

    // 清理资源占位
    daemon_manager.remove_pid_file()?;

    Ok(())
}