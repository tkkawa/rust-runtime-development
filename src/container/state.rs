use std::collections::HashMap;
use std::fs;
use std::{fs::File, path::Path, path::PathBuf};
use std::io::{BufReader, BufWriter, Write};
use chrono::{DateTime, Utc};
// use anyhow::Ok;
use serde::{Deserialize, Serialize};
use tracing::instrument;


#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("failed to open container state file {state_file_path:?}")]
    OpenStateFile {
        state_file_path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse container state file {state_file_path:?}")]
    ParseStateFile {
        state_file_path: PathBuf,
        source: serde_json::Error,
    },
    #[error("failed to write container state file {state_file_path:?}")]
    WriteStateFile {
        state_file_path: PathBuf,
        source: std::io::Error,
    },
}

type Result<T> = std::result::Result<T, StateError>;


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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<DateTime<Utc>>,
}


impl State {
    const STATE_FILE_PATH: &'static str = "state.json";

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
            created: None,
        }
    }

    #[instrument(level = "trace")]
    pub fn save(&self, container_root: &Path) -> Result<()> {
        // let state_file_path = container_root.join(STATE_FILE_PATH);
        // let file = fs::OpenOptions::new()
        //     .read(true)
        //     .write(true)
        //     .append(false)
        //     .create(true)
        //     .truncate(true)
        //     .open(state_file_path)
        //     .expect("Unable to open");
        // serde_json::to_writer(&file, self)?;
        let state_file_path = Self::file_path(container_root);

        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .append(false)
            .create(true)
            .truncate(true)
            .open(&state_file_path)
            .map_err(|err| {
                tracing::error!(
                    state_file_path = ?state_file_path,
                    err = %err,
                    "failed to open container state file",
                );
                StateError::OpenStateFile {
                    state_file_path: state_file_path.to_owned(),
                    source: err,
                }
            })?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, self).map_err(|err| {
            tracing::error!(
                ?state_file_path,
                %err,
                "failed to parse container state file",
            );
            StateError::ParseStateFile {
                state_file_path: state_file_path.to_owned(),
                source: err,
            }
        })?;
        writer.flush().map_err(|err| {
            tracing::error!(
                ?state_file_path,
                %err,
                "failed to write container state file",
            );
            StateError::WriteStateFile {
                state_file_path: state_file_path.to_owned(),
                source: err,
            }
        })?;
        Ok(())
    }

    pub fn load(container_root: &Path) -> Result<Self> {
        // /var/run/youki/${container_number}/state.json
        // let state_file_path = container_root.join(STATE_FILE_PATH);
        // let file = File::open(state_file_path)?;
        // let state: Self = serde_json::from_reader(&file)?;
        let state_file_path = Self::file_path(container_root);
        let state_file = File::open(&state_file_path).map_err(|err| {
            tracing::error!(
                ?state_file_path,
                %err,
                "failed to open container state file",
            );
            StateError::OpenStateFile {
                state_file_path: state_file_path.to_owned(),
                source: err,
            }
        })?;

        let state: Self = serde_json::from_reader(BufReader::new(state_file)).map_err(|err| {
            tracing::error!(
                ?state_file_path,
                %err,
                "failed to parse container state file",
            );
            StateError::ParseStateFile {
                state_file_path: state_file_path.to_owned(),
                source: err,
            }
        })?;
        Ok(state)
    }

    pub fn file_path(container_root: &Path) -> PathBuf {
        container_root.join(Self::STATE_FILE_PATH)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ContainerStatus {
    // The container is being created
    Creating,
    // The runtime has finished the create operation
    Created,
    // The container process has executed the user-specified program but has not exited
    Running,
    // The container process has exited
    Stopped,
    // The container process has paused
    Paused,
}

impl ContainerStatus {
    pub fn can_start(&self) -> bool {
        matches!(self, ContainerStatus::Created)
    }
}
