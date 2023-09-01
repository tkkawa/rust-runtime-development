use std::fs::File;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Root {
    #[serde(default)]
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Linux {
    
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
