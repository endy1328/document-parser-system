use actix_web::{get, web, HttpResponse, Responder};
use uuid::Uuid;
use crate::services::job_queue::JobQueue;
use serde_json::json;

// GET /jobs/{job_id} - 작업 상태 조회
#[get("/jobs/{job_id}")]
pub async fn get_job_status(
    job_queue: web::Data<JobQueue>,
    job_id: web::Path<Uuid>
) -> impl Responder {
    match job_queue.get_job(*job_id).await {
        Ok(Some(job)) => {
            HttpResponse::Ok().json(json!({
                "job_id": job.id.to_string(),
                "status": job.status,
                "progress": job.progress,
                "message": job.message
            }))
        },
        Ok(None) => {
            HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "작업을 찾을 수 없습니다."
            }))
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("작업 상태 조회 중 오류가 발생했습니다: {}", e)
            }))
        }
    }
}

// GET /jobs/{job_id}/result - 작업 결과 조회
#[get("/jobs/{job_id}/result")]
pub async fn get_job_result(
    job_queue: web::Data<JobQueue>,
    job_id: web::Path<Uuid>
) -> impl Responder {
    match job_queue.get_job(*job_id).await {
        Ok(Some(job)) => {
            if let Some(result) = job.result {
                HttpResponse::Ok().json(json!({
                    "job_id": job.id.to_string(),
                    "result": result
                }))
            } else {
                HttpResponse::Accepted().json(json!({
                    "job_id": job.id.to_string(),
                    "status": job.status,
                    "progress": job.progress,
                    "message": "결과가 아직 준비되지 않았습니다."
                }))
            }
        },
        Ok(None) => {
            HttpResponse::NotFound().json(json!({
                "status": "error",
                "message": "작업을 찾을 수 없습니다."
            }))
        },
        Err(e) => {
            HttpResponse::InternalServerError().json(json!({
                "status": "error",
                "message": format!("작업 결과 조회 중 오류가 발생했습니다: {}", e)
            }))
        }
    }
}
