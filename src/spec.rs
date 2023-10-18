use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;
use caps::Capability;
use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct Box {
    #[serde(default)]
    pub height: u64,
    #[serde(default)]
    pub width: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(default)]
    pub uid: u32,
    #[serde(default)]
    pub gid: u32,
    #[serde(default)]
    pub additional_gids: Vec<u32>,
    #[serde(default)]
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Process {
    #[serde(default)]
    pub terminal: bool,
    #[serde(default)]
    pub console_size: Box,
    pub user: User,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub cwd: String,
    #[serde(default)]
    pub no_new_privileges: bool,
    #[serde(default)]
    pub apparmor_profile: String,
    #[serde(default)]
    pub selinux_label: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    #[serde(default)]
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mount {
    #[serde(default)]
    pub destination: PathBuf,
    #[serde(default, rename = "type")]
    pub typ: String,
    #[serde(default)]
    pub source: PathBuf,
    #[serde(default)]
    pub options: Vec<String>,
}

// a is for LinuxDeviceCgroup
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum LinuxDeviceType {
    B,
    C,
    U,
    P,
    A,
}

impl Default for LinuxDeviceType {
    fn default() -> LinuxDeviceType {
        LinuxDeviceType::A
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinuxDeviceCgroup {
    #[serde(default)]
    pub allow: bool,
    #[serde(default, rename = "type")]
    pub typ: LinuxDeviceType,
    pub major: Option<i64>,
    pub minor: Option<i64>,
    #[serde(default)]
    pub access: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinuxMemory {
    pub limit: Option<i64>,
    pub reservation: Option<i64>,
    pub swap: Option<i64>,
    pub kernel: Option<i64>,
    #[serde(rename = "kernelTCP")]
    pub kernel_tcp: Option<i64>,
    pub swappiness: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinuxCPU {
    pub shares: Option<u64>,
    pub quota: Option<i64>,
    pub period: Option<u64>,
    pub realtime_runtime: Option<i64>,
    pub realtime_period: Option<u64>,
    #[serde(default)]
    pub cpus: String,
    #[serde(default)]
    pub mems: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinuxPids {
    #[serde(default)]
    pub limit: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinuxWeightDevice {
    #[serde(default)]
    pub major: i64,
    #[serde(default)]
    pub minor: i64,
    pub weight: Option<u16>,
    pub leaf_weight: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinuxThrottleDevice {
    #[serde(default)]
    pub major: i64,
    #[serde(default)]
    pub minor: i64,
    #[serde(default)]
    pub rate: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinuxBlockIO {
    pub blkio_weight: Option<u16>,
    pub blkio_leaf_weight: Option<u16>,
    #[serde(default)]
    pub blkio_weight_device: Vec<LinuxWeightDevice>,
    #[serde(default)]
    pub blkio_throttle_read_bps_device: Vec<LinuxThrottleDevice>,
    #[serde(default)]
    pub blkio_throttle_write_bps_device: Vec<LinuxThrottleDevice>,
    #[serde(default, rename = "blkioThrottleReadIOPSDevice")]
    pub blkio_throttle_read_iops_device: Vec<LinuxThrottleDevice>,
    #[serde(default, rename = "blkioThrottleWriteIOPSDevice")]
    pub blkio_throttle_write_iops_device: Vec<LinuxThrottleDevice>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinuxHugepageLimit {
    #[serde(default)]
    pub page_size: String,
    #[serde(default)]
    pub limit: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinuxInterfacePriority {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub priority: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinuxNetwork {
    #[serde(rename = "classID")]
    pub class_id: Option<u32>,
    #[serde(default)]
    pub priorities: Vec<LinuxInterfacePriority>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinuxResources {
    #[serde(default)]
    pub devices: Vec<LinuxDeviceCgroup>,
    #[serde(default)]
    pub disable_oom_killer: bool,
    pub oom_score_adj: Option<i32>,
    pub memory: Option<LinuxMemory>,
    pub cpu: Option<LinuxCPU>,
    pub pids: Option<LinuxPids>,
    #[serde(rename = "blockIO")]
    pub block_io: Option<LinuxBlockIO>,
    #[serde(default)]
    pub hugepage_limits: Vec<LinuxHugepageLimit>,
    pub network: Option<LinuxNetwork>,
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


#[derive(Serialize, Deserialize, Debug)]
pub struct LinuxNamespace {
    #[serde(rename = "type")]
    pub typ: LinuxNamespaceType,
    #[serde(default)]
    pub path: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinuxDevice {
    #[serde(default)]
    pub path: String,
    #[serde(rename = "type")]
    pub typ: LinuxDeviceType,
    #[serde(default)]
    pub major: u64,
    #[serde(default)]
    pub minor: u64,
    pub file_mode: Option<u32>,
    pub uid: Option<u32>,
    pub gid: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Linux {
    pub resources: Option<LinuxResources>,
    #[serde(default)]
    pub namespaces: Vec<LinuxNamespace>,
    #[serde(default)]
    pub rootfs_propagation: String,
    #[serde(default)]
    pub devices: Vec<LinuxDevice>,
    #[serde(default)]
    pub mount_label: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Spec{
    pub root: Root,
    pub process: Process,
    #[serde(default)]
    pub hostname: String,
    #[serde(default)]
    pub mounts: Vec<Mount>,
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
