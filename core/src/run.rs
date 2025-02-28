use crate::blocks::block::BlockType;
use crate::project::Project;
use crate::stores::{sqlite::SQLiteStore, store::Store};
use crate::utils;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{to_string_pretty, Value};
use std::collections::HashMap;
use std::str::FromStr;

/// BlockExecution represents the execution of a block:
/// - `env` used
/// - `value` returned by successful execution
/// - `error` message returned by a failed execution
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct BlockExecution {
    // pub env: Env,
    pub value: Option<Value>,
    pub error: Option<String>,
}

pub type Credentials = HashMap<String, String>;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct RunConfig {
    pub blocks: HashMap<String, Value>,
}

impl RunConfig {
    pub fn config_for_block(&self, name: &str) -> Option<&Value> {
        self.blocks.get(name)
    }

    pub fn concurrency_for_block(&self, block_type: BlockType, name: &str) -> usize {
        let block_config = self.config_for_block(name);

        if let Some(block_config) = block_config {
            if let Some(concurrency) = block_config.get("concurrency") {
                if let Some(concurrency) = concurrency.as_u64() {
                    return concurrency as usize;
                }
            }
        }

        // Default concurrency parameters
        match block_type {
            BlockType::Input => 64,
            BlockType::Data => 64,
            BlockType::Code => 64,
            BlockType::LLM => 8,
            BlockType::Map => 64,
            BlockType::Reduce => 64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Running,
    Succeeded,
    Errored,
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::Running => "running".to_string(),
            Status::Succeeded => "succeeded".to_string(),
            Status::Errored => "errored".to_string(),
        }
    }
}

impl FromStr for Status {
    type Err = utils::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "running" => Ok(Status::Running),
            "succeeded" => Ok(Status::Succeeded),
            "errored" => Ok(Status::Errored),
            _ => Err(utils::ParseError::with_message("Unknown Status"))?,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockStatus {
    pub block_type: BlockType,
    pub name: String,
    pub status: Status,
    pub success_count: usize,
    pub error_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RunStatus {
    run: Status,
    blocks: Vec<BlockStatus>,
}

impl RunStatus {
    pub fn set_block_status(&mut self, status: BlockStatus) {
        match self
            .blocks
            .iter()
            .position(|s| (s.block_type == status.block_type && s.name == status.name))
        {
            Some(i) => {
                let _ = std::mem::replace(&mut self.blocks[i], status);
            }
            None => {
                self.blocks.push(status);
            }
        }
    }

    pub fn set_run_status(&mut self, status: Status) {
        self.run = status;
    }

    pub fn run_status(&self) -> Status {
        self.run.clone()
    }
}

/// Execution represents the full execution of an app on input data.
#[derive(PartialEq, Debug, Serialize, Clone)]
pub struct Run {
    run_id: String,
    created: u64,
    app_hash: String,
    config: RunConfig,
    status: RunStatus,
    // List of blocks (in order with name) and their execution.
    // The outer vector represents blocks
    // The inner-outer vector represents inputs
    // The inner-inner vector represents mapped outputs
    // If execution was interrupted by errors, the non-executed block won't be present. If a block
    // on a particular Env was not executed due to a conditional execution, its BlockExecution will
    // be present but both output and error will be None.
    // TODO(spolu): note that there is a lot of repetition here in particular through the env
    // variables, will need to be revisited but that's a fair enough starting point.
    pub traces: Vec<((BlockType, String), Vec<Vec<BlockExecution>>)>,
}

impl Run {
    pub fn new(app_hash: &str, config: RunConfig) -> Self {
        Run {
            run_id: utils::new_id(),
            created: utils::now(),
            app_hash: app_hash.to_string(),
            config,
            status: RunStatus {
                run: Status::Running,
                blocks: vec![],
            },
            traces: vec![],
        }
    }

    /// Creates a new Run object in memory from raw data (used by Store implementations)
    pub fn new_from_store(
        run_id: &str,
        created: u64,
        app_hash: &str,
        config: &RunConfig,
        status: &RunStatus,
        traces: Vec<((BlockType, String), Vec<Vec<BlockExecution>>)>,
    ) -> Self {
        Run {
            run_id: run_id.to_string(),
            created,
            app_hash: app_hash.to_string(),
            config: config.clone(),
            status: status.clone(),
            traces,
        }
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    pub fn created(&self) -> u64 {
        self.created
    }

    pub fn app_hash(&self) -> &str {
        &self.app_hash
    }

    pub fn config(&self) -> &RunConfig {
        &self.config
    }

    pub fn status(&self) -> &RunStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: RunStatus) {
        self.status = status;
    }

    pub fn set_run_status(&mut self, status: Status) {
        self.status.run = status;
    }

    pub fn set_block_status(&mut self, status: BlockStatus) {
        self.status.set_block_status(status);
    }
}

pub async fn cmd_inspect(run_id: &str, block_type: BlockType, block_name: &str) -> Result<()> {
    let root_path = utils::init_check().await?;
    let store = SQLiteStore::new(root_path.join("store.sqlite"))?;
    store.init().await?;
    let project = Project::new_from_id(1);

    let mut run_id = run_id.to_string();

    if run_id == "latest" {
        run_id = match store.latest_run_id(&project).await? {
            Some(run_id) => run_id,
            None => Err(anyhow!("No run found, the app was never executed"))?,
        };
        utils::info(&format!("Latest run is `{}`", run_id));
    }

    let run = match store
        .load_run(
            &project,
            &run_id,
            Some(Some((block_type, block_name.to_string()))),
        )
        .await?
    {
        Some(r) => r,
        None => Err(anyhow!("Run with id {} not found", run_id))?,
    };

    let mut found = false;
    run.traces.iter().for_each(|((t, n), input_executions)| {
        if n == block_name && *t == block_type {
            input_executions
                .iter()
                .enumerate()
                .for_each(|(input_idx, map_executions)| {
                    map_executions
                        .iter()
                        .enumerate()
                        .for_each(|(map_idx, execution)| {
                            found = true;
                            utils::info(&format!(
                                "Execution: input_idx={}/{} map_idx={}/{}",
                                input_idx,
                                input_executions.len(),
                                map_idx,
                                map_executions.len()
                            ));
                            match execution.value.as_ref() {
                                Some(v) => println!("{}", to_string_pretty(v).unwrap()),
                                None => {}
                            }
                            match execution.error.as_ref() {
                                Some(e) => utils::error(&format!("Error: {}", e)),
                                None => {}
                            }
                        });
                });
        }
    });

    if !found {
        Err(anyhow!(
            "Block `{} {}` not found in run `{}`",
            block_type.to_string(),
            block_name,
            run_id
        ))?;
    }

    Ok(())
}

pub async fn cmd_list() -> Result<()> {
    let root_path = utils::init_check().await?;
    let store = SQLiteStore::new(root_path.join("store.sqlite"))?;
    store.init().await?;
    let project = Project::new_from_id(1);

    store
        .all_runs(&project)
        .await?
        .iter()
        .for_each(|(run_id, created, app_hash, _config)| {
            utils::info(&format!(
                "Run: {} app_hash={} created={}",
                run_id,
                app_hash,
                utils::utc_date_from(*created),
            ));
        });

    Ok(())
}
