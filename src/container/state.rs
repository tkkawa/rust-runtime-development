use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContainerStatus {
    Creating,
}

impl ContainerStatus {

}
