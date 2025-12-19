use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOperation {
    pub id: Uuid,
    pub table_name: String,
    pub operation: Operation,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequest {
    pub tenant_id: String,
    pub client_id: Uuid,
    pub last_version: i64,
    pub operations: Vec<SyncOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResponse {
    pub operations: Vec<SyncOperation>,
    pub current_version: i64,
    pub conflicts: Vec<Conflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub operation_id: Uuid,
    pub reason: String,
    pub server_version: serde_json::Value,
}
