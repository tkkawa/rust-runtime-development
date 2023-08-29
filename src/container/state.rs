use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

pub struct State {

}

impl State {
    pub fn new(
        container_id: &str,
        status: ContainerStatus,
        pid: Option<i32>,
        bundle: &str,
    ) -> Self {
        Self {
            
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContainerStatus {
    Creating,
}

impl ContainerStatus {

}
