use std::collections::HashMap;
use std::fs;
use std::{fs::File, path::PathBuf};

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};

const STATE_FILE_PATH: &str = "state.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct State {
    // Version is the version of the specification that is supported.
    pub oci_version: String,
    // ID is the container ID
    pub id: String,
    // Status is the runtime status of the container.
    pub status: ContainerStatus,
    // Pid is the process ID for the container process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<i32>,
    // Bundle is the path to the container's bundle directory.
    pub bundle: String,
    // Annotations are key values associated with the container.
    pub annotations: HashMap<String, String>,
}

impl State {
    pub fn new(
        container_id: &str,
        status: ContainerStatus,
        pid: Option<i32>,
        bundle: &str,
    ) -> Self {
        Self {
            oci_version: "v1.0.2".to_string(),
            id: container_id.to_string(),
            status,
            pid,
            bundle: bundle.to_string(),
            annotations: HashMap::default(),
        }
    }

    pub fn save(&self, container_root: &PathBuf) -> Result<()> {
        let state_file_path = container_root.join(STATE_FILE_PATH);
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .append(false)
            .create(true)
            .truncate(true)
            .open(state_file_path)
            .expect("Unable to open");
        serde_json::to_writer(&file, self)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContainerStatus {
    Creating,
}

impl ContainerStatus {

}
