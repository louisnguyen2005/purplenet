use anyhow::{bail, Result};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::IpAddr;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkScope {
    pub schema_version: u32,
    pub id: String,
    pub owner: String,
    pub environment: String,
    pub allowed_cidrs: Vec<IpNet>,
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub excluded_ips: Vec<IpAddr>,
    pub valid_from_unix: u64,
    pub valid_until_unix: u64,
    pub max_mock_events: u32,
    pub allowed_scenarios: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub id: String,
    pub ip: IpAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    pub schema_version: u32,
    pub id: String,
    pub scope_id: String,
    pub scope_sha256: String,
    pub scenario_id: String,
    pub approved_by: String,
    pub valid_from_unix: u64,
    pub valid_until_unix: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub schema_version: u32,
    pub id: String,
    pub description: String,
    pub target: IpAddr,
    pub target_asset_id: String,
    pub mock_events: Vec<MockEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MockEvent {
    pub id: String,
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Approved,
    Ready,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ObservedEvent {
    pub id: String,
    pub label: String,
    pub source: String,
    pub observed_unix: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Run {
    pub schema_version: u32,
    pub id: String,
    pub scope_id: String,
    pub scope_sha256: String,
    pub approval_id: String,
    pub scenario_id: String,
    pub scenario_sha256: String,
    pub actor: String,
    pub target: IpAddr,
    pub state: RunState,
    pub created_unix: u64,
    pub updated_unix: u64,
    pub events: Vec<ObservedEvent>,
}

pub fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
}

fn valid_person(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 120
}

impl NetworkScope {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 || !valid_id(&self.id) {
            bail!("invalid scope version or ID");
        }
        if !valid_person(&self.owner) || !valid_id(&self.environment) {
            bail!("invalid owner or environment");
        }
        if self.allowed_cidrs.is_empty() || self.allowed_cidrs.len() > 32 {
            bail!("scope needs 1..=32 CIDRs");
        }
        if self.assets.is_empty() || self.assets.len() > 256 {
            bail!("scope needs 1..=256 named assets");
        }
        let mut asset_ids = std::collections::HashSet::new();
        let mut asset_ips = std::collections::HashSet::new();
        for asset in &self.assets {
            if !valid_id(&asset.id)
                || !asset_ids.insert(&asset.id)
                || !asset_ips.insert(asset.ip)
                || !self
                    .allowed_cidrs
                    .iter()
                    .any(|cidr| cidr.contains(&asset.ip))
                || self.excluded_ips.contains(&asset.ip)
            {
                bail!("invalid, duplicate, or out-of-scope asset");
            }
        }
        if self.valid_from_unix >= self.valid_until_unix {
            bail!("invalid scope window");
        }
        if self.max_mock_events == 0 || self.max_mock_events > 100 {
            bail!("max_mock_events must be 1..=100");
        }
        if self.allowed_scenarios.is_empty()
            || self.allowed_scenarios.iter().any(|id| !valid_id(id))
        {
            bail!("invalid allowed_scenarios");
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<String> {
        let bytes = serde_json::to_vec(self)?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }
}

impl Approval {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !valid_id(&self.id)
            || !valid_id(&self.scope_id)
            || !valid_id(&self.scenario_id)
            || !valid_person(&self.approved_by)
            || self.scope_sha256.len() != 64
            || !self.scope_sha256.bytes().all(|b| b.is_ascii_hexdigit())
            || self.valid_from_unix >= self.valid_until_unix
        {
            bail!("invalid approval");
        }
        Ok(())
    }
}

impl Scenario {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !valid_id(&self.id)
            || self.description.len() > 500
            || !valid_id(&self.target_asset_id)
            || self.mock_events.is_empty()
            || self.mock_events.len() > 100
        {
            bail!("invalid scenario");
        }
        let mut ids = std::collections::HashSet::new();
        for event in &self.mock_events {
            if !valid_id(&event.id)
                || event.label.trim().is_empty()
                || event.label.len() > 200
                || !ids.insert(&event.id)
            {
                bail!("invalid or duplicate mock event ID");
            }
        }
        Ok(())
    }

    pub fn digest(&self) -> Result<String> {
        Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(self)?)))
    }
}
