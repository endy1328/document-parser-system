use crate::models::job::{Job, JobStatus};
use redis::{AsyncCommands, Client};
use uuid::Uuid;
use serde_json;

#[derive(Clone)]
pub struct JobQueue {
    pub redis_url: String,
    pub client: Client,
}

impl JobQueue {
    pub fn new(redis_url: &str) -> Self {
        Self { 
            redis_url: redis_url.to_string(), 
            client: Client::open(redis_url).unwrap() 
        }
    }

    pub async fn enqueue_job(&self, job: &Job) -> redis::RedisResult<String> {    
        let job_json = serde_json::to_string(job).map_err(|e| {
            redis::RedisError::from((redis::ErrorKind::IoError, "Failed to serialize job", e.to_string()))
        })?;

        let mut con = self.client.get_async_connection().await?;
        let job_key = format!("jobs:{}", job.id.to_string());
        
        // Redis에 작업 정보 저장
        let _: () = con.set(&job_key, &job_json).await?;
        
        // 작업 ID를 큐에 추가
        let _: () = con.lpush("jobs:queue", job.id.to_string()).await?;
        
        Ok("OK".to_string())
    }

    pub async fn update_job_status(&self, job_id: Uuid, status: JobStatus, progress: u8, message: Option<String>) -> redis::RedisResult<String> {
        let mut con = self.client.get_async_connection().await?;
        let job_key = format!("jobs:{}", job_id.to_string());
        
        // 기존 작업 정보 가져오기
        let job_json: String = con.get(&job_key).await?;
        
        // 작업 정보 업데이트
        match serde_json::from_str::<Job>(&job_json) {
            Ok(mut job) => {
                job.status = status;
                job.progress = progress;
                if let Some(msg) = message {
                    job.message = Some(msg);
                }
                
                let job_json = serde_json::to_string(&job).map_err(|e| {
                    redis::RedisError::from((redis::ErrorKind::IoError, "Failed to serialize job", e.to_string()))
                })?;
                
                let _: () = con.set(format!("jobs:{}", job_id.to_string()), job_json).await?;
                Ok("OK".to_string())
            },
            Err(e) => Err(redis::RedisError::from((redis::ErrorKind::IoError, "Failed to deserialize job", e.to_string())))
        }
    }

    pub async fn get_job(&self, job_id: Uuid) -> redis::RedisResult<Option<Job>> {
        let mut con = self.client.get_async_connection().await?;
        
        // Redis에서 작업 조회
        let job_key = format!("jobs:{}", job_id.to_string());
        let job_json: Option<String> = con.get(&job_key).await?;
        
        match job_json {
            Some(json_str) => {
                // JSON 문자열을 Job 객체로 변환
                match serde_json::from_str::<Job>(&json_str) {
                    Ok(job) => Ok(Some(job)),
                    Err(_) => Err(redis::RedisError::from((redis::ErrorKind::IoError, "Invalid job data format")))
                }
            },
            None => Ok(None) // 작업 없음
        }
    }
}
