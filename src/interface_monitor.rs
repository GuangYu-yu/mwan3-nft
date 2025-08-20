use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::process::Command;
use anyhow::Result;

use crate::config::Config;
use crate::load_balancer::LoadBalancer;

pub struct InterfaceMonitor {
    config: Arc<RwLock<Config>>,
    load_balancer: Arc<LoadBalancer>,
}

impl InterfaceMonitor {
    pub fn new(config: Arc<RwLock<Config>>, load_balancer: Arc<LoadBalancer>) -> Self {
        Self {
            config,
            load_balancer,
        }
    }
    
    pub async fn start(&self) -> Result<()> {
        // 启动接口监控占位
        self.monitor_interfaces().await
    }
    
    async fn monitor_interfaces(&self) -> Result<()> {
        // 使用 ip monitor link 监控接口状态变化占位
        let mut cmd = Command::new("ip")
            .args(&["monitor", "link"])
            .stdout(std::process::Stdio::piped())
            .spawn()?;
        
        // 处理监控输出占位
        if let Some(stdout) = cmd.stdout.take() {
            self.process_monitor_output(stdout).await?;
        }
        
        Ok(())
    }
    
    async fn process_monitor_output(&self, stdout: tokio::process::ChildStdout) -> Result<()> {
        use tokio::io::{AsyncBufReadExt, BufReader};
        
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        
        while let Some(line) = lines.next_line().await? {
            self.parse_interface_event(&line).await?;
        }
        
        Ok(())
    }
    
    async fn parse_interface_event(&self, line: &str) -> Result<()> {
        // 解析接口事件占位
        // 示例: "2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500"
        
        if line.contains("UP") {
            // 接口上线
            if let Some(interface_name) = self.extract_interface_name(line) {
                self.load_balancer.handle_interface_change(&interface_name, true).await?;
            }
        } else if line.contains("DOWN") {
            // 接口下线
            if let Some(interface_name) = self.extract_interface_name(line) {
                self.load_balancer.handle_interface_change(&interface_name, false).await?;
            }
        }
        
        Ok(())
    }
    
    fn extract_interface_name(&self, line: &str) -> Option<String> {
        // 从监控输出中提取接口名称占位
        // 简单的解析逻辑
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[1].trim_end_matches(':');
            Some(name.to_string())
        } else {
            None
        }
    }
}