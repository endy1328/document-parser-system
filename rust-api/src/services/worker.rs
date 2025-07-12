use std::time::Duration;
use uuid::Uuid;
use log::{info, error};
use tokio::time;

use crate::models::job::{JobStatus, JobResult};
use crate::models::document::FileType;
use crate::services::job_queue::JobQueue;
use crate::services::parser::DocumentParser;

pub struct BackgroundWorker {
    job_queue: JobQueue,
    parser: DocumentParser,
    poll_interval: Duration,
}

impl BackgroundWorker {
    pub fn new(job_queue: JobQueue, poll_interval_secs: u64) -> Self {
        Self {
            job_queue,
            parser: DocumentParser::new(),
            poll_interval: Duration::from_secs(poll_interval_secs),
        }
    }

    pub async fn start(&self) {
        info!("백그라운드 워커 시작됨");
        
        loop {
            // 주기적으로 큐에서 작업 확인
            if let Err(e) = self.process_next_job().await {
                error!("작업 처리 중 오류 발생: {}", e);
            }
            
            // 지정된 간격으로 대기
            time::sleep(self.poll_interval).await;
        }
    }

    async fn process_next_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Redis에서 큐의 다음 작업 ID 가져오기
        let client = redis::Client::open(self.job_queue.redis_url.as_str())?;
        let mut con = client.get_async_connection().await?;
        
        // jobs:queue에서 작업 ID 가져오기 (pop)
        // rpop의 count 매개변수는 None으로 지정 (단일 항목 pop)
        let job_id: Option<String> = redis::AsyncCommands::rpop(&mut con, "jobs:queue", None).await?;
        
        if let Some(id_str) = job_id {
            let job_id = match Uuid::parse_str(&id_str) {
                Ok(id) => id,
                Err(_) => return Err("유효하지 않은 작업 ID".into()),
            };
            
            info!("작업 처리 시작: {}", job_id);
            
            // 작업 정보 가져오기
            match self.job_queue.get_job(job_id).await? {
                Some(job) => {
                    // 작업 상태를 "처리 중"으로 업데이트
                    self.job_queue.update_job_status(
                        job_id,
                        JobStatus::Processing,
                        0,
                        Some(String::from("파싱 작업 시작"))
                    ).await?;
                    
                    // 파일 경로 구성 (uploads/{filename})
                    // job 메시지에서 파일 정보 추출
                    let empty_message = String::from(""); // 빈 문자열을 미리 생성
                    let message = job.message.as_ref().unwrap_or(&empty_message);
                    
                    // 파일명 추출
                    let filename = if let Some(idx) = message.find("(") {
                        message[0..idx].trim().to_string()
                    } else {
                        String::from("unknown.pdf")  // 기본값
                    };
                    
                    // 파일 경로 구성
                    let filepath = format!("uploads/{}", filename);
                    
                    // 파일 타입 추출
                    let filetype = FileType::from_filename(&filename);
                    let file_type_str = filetype.to_string().to_string();  // &str을 String으로 변환
                    
                    // 문서 파싱 시작
                    match self.parser.parse_document(&filepath, filetype).await {
                        Ok(content) => {
                            // 파싱 성공 - 결과 저장
                            let result = JobResult {
                                filename: filename.clone(),  // 실제 파일명 사용
                                file_type: file_type_str,  // 미리 변환해둘 문자열 사용
                                content: serde_json::to_value(content).unwrap_or(serde_json::json!({})),
                                metadata: serde_json::json!({
                                    "processed_at": chrono::Utc::now().to_rfc3339(),
                                    "filepath": filepath
                                }),
                            };
                            
                            // 작업 완료 상태로 업데이트 및 결과 저장
                            let mut updated_job = job.clone();
                            updated_job.status = JobStatus::Completed;
                            updated_job.progress = 100;
                            updated_job.message = Some(String::from("파싱 작업 완료"));
                            updated_job.result = Some(result);
                            
                            // Redis에 최종 상태 저장
                            let job_json = serde_json::to_string(&updated_job)?;
                            let job_key = format!("jobs:{}", job_id);
                            let _: () = redis::AsyncCommands::set(&mut con, &job_key, job_json).await?;
                            
                            info!("작업 완료: {}", job_id);
                        },
                        Err(e) => {
                            // 파싱 실패
                            self.job_queue.update_job_status(
                                job_id,
                                JobStatus::Failed,
                                100,
                                Some(format!("파싱 중 오류 발생: {}", e))
                            ).await?;
                            
                            error!("파싱 실패 [{}]: {}", job_id, e);
                        }
                    }
                },
                None => {
                    error!("큐에 등록된 작업 ID가 존재하지 않습니다: {}", job_id);
                }
            }
        }
        
        Ok(())
    }
    
    // 진행 상황 시뮬레이션 (실제 구현 시 파싱 단계에 따른 진행률 업데이트 필요)
    async fn simulate_progress(&self, job_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        for progress in (0..=100).step_by(10) {
            self.job_queue.update_job_status(
                job_id,
                JobStatus::Processing,
                progress,
                Some(format!("문서 처리 중... {}%", progress))
            ).await?;
            
            time::sleep(Duration::from_millis(500)).await;
        }
        Ok(())
    }
}
