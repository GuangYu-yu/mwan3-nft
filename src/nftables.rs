use std::collections::HashMap;
use tokio::process::Command;
use anyhow::Result;

use crate::config::{Policy, Interface};

pub struct NftablesManager {
    table_name: String,
}

impl NftablesManager {
    pub fn new() -> Self {
        Self {
            table_name: "mwan3".to_string(),
        }
    }
    
    pub async fn initialize(&self) -> Result<()> {
        // 初始化 nftables 表和链占位
        self.create_table().await?;
        self.create_chains().await?;
        Ok(())
    }
    
    async fn create_table(&self) -> Result<()> {
        // 创建 nftables 表占位
        let cmd = format!("add table inet {}", self.table_name);
        self.execute_nft_command(&cmd).await?;
        Ok(())
    }
    
    async fn create_chains(&self) -> Result<()> {
        // 创建基础链占位
        let chains = vec![
            "mwan3_hook",
            "mwan3_connected",
            "mwan3_track",
            "mwan3_policy",
            "mwan3_rules",
        ];
        
        for chain in chains {
            let cmd = format!("add chain inet {} {}", self.table_name, chain);
            self.execute_nft_command(&cmd).await?;
        }
        
        // 创建 hook 链
        let hook_cmd = format!(
            "add chain inet {} mwan3_hook {{ type route hook output priority mangle; }}",
            self.table_name
        );
        self.execute_nft_command(&hook_cmd).await?;
        
        Ok(())
    }
    
    pub async fn update_rules(&self, interface: &str) -> Result<()> {
        // 更新路由规则占位
        self.clear_interface_rules(interface).await?;
        self.add_interface_rules(interface).await?;
        Ok(())
    }
    
    async fn clear_interface_rules(&self, interface: &str) -> Result<()> {
        // 清除接口相关规则占位
        let cmd = format!(
            "flush chain inet {} mwan3_policy",
            self.table_name
        );
        self.execute_nft_command(&cmd).await?;
        Ok(())
    }
    
    async fn add_interface_rules(&self, interface: &str) -> Result<()> {
        // 添加接口规则占位
        let cmd = format!(
            "add rule inet {} mwan3_policy oif {} mark set 0x{:x}",
            self.table_name, interface, 1
        );
        self.execute_nft_command(&cmd).await?;
        Ok(())
    }
    
    pub async fn setup_round_robin(&self, interfaces: &[String], policy: &Policy) -> Result<()> {
        // 设置轮询规则占位
        self.clear_policy_rules().await?;
        
        for (i, interface) in interfaces.iter().enumerate() {
            let weight = i + 1;
            let cmd = format!(
                "add rule inet {} mwan3_policy numgen random mod {} vmap {{ {} : mark set 0x{:x} }}",
                self.table_name, interfaces.len(), i, weight
            );
            self.execute_nft_command(&cmd).await?;
        }
        
        Ok(())
    }
    
    pub async fn setup_failover(&self, primary: &str) -> Result<()> {
        // 设置故障转移规则占位
        self.clear_policy_rules().await?;
        
        let cmd = format!(
            "add rule inet {} mwan3_policy mark set 0x1",
            self.table_name
        );
        self.execute_nft_command(&cmd).await?;
        
        Ok(())
    }
    
    async fn clear_policy_rules(&self) -> Result<()> {
        // 清除策略规则占位
        let cmd = format!("flush chain inet {} mwan3_policy", self.table_name);
        self.execute_nft_command(&cmd).await?;
        Ok(())
    }
    
    pub async fn setup_interface_sets(&self, interface: &Interface) -> Result<()> {
        // 设置接口相关的 sets 规则占位
        for set_name in &interface.nftables_sets {
            let cmd = format!(
                "add rule inet {} mwan3_rules ip saddr @{} mark set 0x{:x}",
                self.table_name, set_name, interface.mark
            );
            self.execute_nft_command(&cmd).await?;
        }
        
        Ok(())
    }
    
    pub async fn update_interface_mark(&self, interface: &str, mark: u32, enabled: bool) -> Result<()> {
        // 更新接口标记占位
        if enabled {
            let cmd = format!(
                "add rule inet {} mwan3_track oif {} mark set 0x{:x}",
                self.table_name, interface, mark
            );
            self.execute_nft_command(&cmd).await?;
        } else {
            // 移除接口标记规则
            self.remove_interface_mark_rules(interface).await?;
        }
        
        Ok(())
    }
    
    async fn remove_interface_mark_rules(&self, interface: &str) -> Result<()> {
        // 移除接口标记规则占位
        // 这里需要更复杂的规则删除逻辑
        Ok(())
    }
    
    async fn execute_nft_command(&self, command: &str) -> Result<()> {
        // 执行 nft 命令占位
        let output = Command::new("nft")
            .arg(command)
            .output()
            .await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("nft command failed: {}", stderr));
        }
        
        Ok(())
    }
    
    pub async fn get_table_rules(&self) -> Result<String> {
        // 获取表规则占位
        let output = Command::new("nft")
            .args(&["list", "table", "inet", &self.table_name])
            .output()
            .await?;
        
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
    
    pub async fn backup_rules(&self, file_path: &str) -> Result<()> {
        // 备份规则占位
        let rules = self.get_table_rules().await?;
        tokio::fs::write(file_path, rules).await?;
        Ok(())
    }
    
    pub async fn restore_rules(&self, file_path: &str) -> Result<()> {
        // 恢复规则占位
        let rules = tokio::fs::read_to_string(file_path).await?;
        let mut child = Command::new("nft")
            .args(&["-f", "-"])
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(rules.as_bytes()).await?;
        }
        
        child.wait().await?;
        Ok(())
    }
}