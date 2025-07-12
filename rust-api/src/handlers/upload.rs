use actix_web::{post, web, HttpResponse, Error};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use std::io::Write;
use std::fs;
use serde_json::json;
use uuid::Uuid;

use crate::models::document::FileType;
use crate::services::job_queue::JobQueue;
use crate::models::job::{Job, JobStatus};

#[post("/upload")]
pub async fn upload(
    job_queue: web::Data<JobQueue>,
    mut payload: Multipart
) -> Result<HttpResponse, Error> {
    // uploads 폴더 생성
    let upload_dir = "uploads";
    fs::create_dir_all(upload_dir).ok();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        
        // 파일 정보 추출
        let filename = field.content_disposition()
            .get_filename()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("file_{}", Uuid::new_v4()));
        
        let job_id = Uuid::new_v4();
        let filepath = format!("{}/{}", upload_dir, filename);
        let file_type = FileType::from_filename(&filename);

        // 지원하지 않는 파일 형식 체크
        if matches!(file_type, FileType::Unknown) {
            return Ok(HttpResponse::BadRequest().json(json!({
                "status": "error",
                "message": "지원하지 않는 파일 형식입니다.",
                "supported_formats": ["pdf", "docx", "xlsx", "txt", "md"]
            })));
        }

        // 파일 저장
        let mut f = fs::File::create(&filepath)?;
        while let Some(chunk) = field.next().await {
            let chunk = chunk?;
            f.write_all(&chunk)?;
        }

        // 작업 큐에 등록
        let job = Job {
            id: job_id,
            status: JobStatus::Queued,
            progress: 0,
            message: Some(format!("{}({}) 파일 처리 대기 중", filename, file_type.to_string())),
            result: None,
        };
        
        // 작업 큐에 등록 시도
        match job_queue.enqueue_job(&job).await {
            Ok(_) => {
                // 작업 등록 성공 - 즉시 job_id 반환
                return Ok(HttpResponse::Accepted().json(json!({
                    "status": "accepted",
                    "job_id": job_id.to_string(),
                    "filename": filename,
                    "file_type": file_type.to_string(),
                    "message": "파일이 업로드되어 처리 대기열에 등록되었습니다."
                })));
            },
            Err(e) => {
                // 작업 등록 실패
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "status": "error",
                    "message": format!("작업 등록 중 오류가 발생했습니다: {}", e)
                })));
            }
        }
    }

    Ok(HttpResponse::BadRequest().json(json!({
        "status": "error",
        "message": "파일이 업로드되지 않았습니다."
    })))
}
