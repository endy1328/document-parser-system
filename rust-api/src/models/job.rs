use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum JobStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub id: Uuid,
    pub status: JobStatus,
    pub progress: u8, // 0~100
    pub message: Option<String>,
    pub result: Option<JobResult>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobResult {
    pub filename: String,
    pub file_type: String,
    pub content: serde_json::Value,
    pub metadata: serde_json::Value,
}
