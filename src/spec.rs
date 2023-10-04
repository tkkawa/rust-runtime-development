use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    #[serde(default)]
    pub path: PathBuf,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinuxRlimits {
    #[serde(rename = "type")]
    pub typ: LinuxRlimitType,
    #[serde(default)]
    pub soft: u64,
    #[serde(default)]
    pub hard: u64,
}

// https://containers.github.io/oci-spec-rs/oci_spec/runtime/enum.LinuxRlimitType.html
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LinuxRlimitType {
    RlimitCpu,
    RlimitFsize,
    RlimitData,
    RlimitStack,
    RlimitCore,
    RlimitRss,
    RlimitNproc,
    RlimitNofile,
    RlimitMemlock,
    RlimitAs,
    RlimitLocks,
    RlimitSigpending,
    RlimitMsgqueue,
    RlimitNice,
    RlimitRtprio,
    RlimitRttime,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LinuxNamespaceType {
    Mount = 0x00020000,
    Cgroup = 0x02000000,
    Uts = 0x04000000,
    Ipc = 0x08000000,
    User = 0x10000000,
    Pid = 0x20000000,
    Network = 0x40000000,
}

#[derive(Debug, Clone)]
pub struct LinuxCapabilityType {
    pub cap: Capability,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LinuxCapabilities {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bounding: Vec<LinuxCapabilityType>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub effective: Vec<LinuxCapabilityType>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inheritable: Vec<LinuxCapabilityType>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permitted: Vec<LinuxCapabilityType>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ambient: Vec<LinuxCapabilityType>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinuxNamespace {
    #[serde(rename = "type")]
    pub typ: LinuxNamespaceType,
    #[serde(default)]
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Linux {
    #[serde(default)]
    pub namespaces: Vec<LinuxNamespace>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec{
    pub root: Root,
    pub linux: Option<Linux>,
}

impl Spec {
    pub fn load(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let mut spec: Spec = serde_json::from_reader(&file)?;
        spec.root.path = std::fs::canonicalize(spec.root.path)?;
        Ok(spec)
    }
}
