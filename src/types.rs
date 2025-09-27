use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Combo {
    pub name: String,
    pub commands: Vec<String>,
    pub last_used: i64,
}

pub enum SaveOption {
    Edit,
    SaveAsScript,
    SaveAsFunction,
    SaveAsCombo,
}
