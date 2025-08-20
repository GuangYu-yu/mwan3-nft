use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    pub interfaces: Vec<Interface>,
    pub policies: Vec<Policy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub policy: String,
    #[serde(rename = "udp-race")]
    pub udp_race: bool,
    pub mptcp: bool,
    pub tfo: bool,
    #[serde(rename = "health-check")]
    pub health_check: HealthCheckConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    pub name: String,
    #[serde(rename = "interface-name")]
    pub interface_name: String,
    pub weight: u32,
    pub mark: u32,
    pub enabled: bool,
    #[serde(rename = "nftables-sets")]
    pub nftables_sets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    #[serde(rename = "type")]
    pub policy_type: String,
    pub interfaces: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub timeout: u64,
    pub interval: u64,
    pub url: String,
    #[serde(rename = "fail-threshold")]
    pub fail_threshold: u32,
    #[serde(rename = "succ-threshold")]
    pub succ_threshold: u32,
}

impl Config {
    pub async fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path).await?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
    
    pub async fn reload(&mut self, path: &str) -> Result<()> {
        let new_config = Self::load(path).await?;
        *self = new_config;
        Ok(())
    }
    
    pub fn validate(&self) -> Result<()> {
        // 配置验证逻辑占位
        Ok(())
    }
}