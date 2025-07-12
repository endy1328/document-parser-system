use actix_web::{get, web, HttpResponse, Responder};
use uuid::Uuid;
use std::fs;
use std::path::Path;
use crate::services::job_queue::JobQueue;
use serde_json::json;

// GET /download/original/{job_id} - 원본 파일 다운로드
#[get("/download/original/{job_id}")]
pub async fn download_original(
    job_queue: web::Data<JobQueue>,
    job_id: web::Path<Uuid>
) -> impl Responder {
    // 작업 정보 조회
    match job_queue.get_job(*job_id).await {
        Ok(Some(job)) => {
            if let Some(result) = &job.result {
                // 원본 파일 경로 추출
                let filename = &result.filename;
                let file_path = format!("uploads/{}", filename);
                
                // 파일 존재 확인
                if Path::new(&file_path).exists() {
                    // 파일 읽기
                    match fs::read(&file_path) {
                        Ok(file_bytes) => {
                            // 파일 타입 확인
                            let content_type = match Path::new(&file_path).extension().and_then(|ext| ext.to_str()) {
                                Some("pdf") => "application/pdf",
                                Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
                                Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                                Some("txt") => "text/plain",
                                Some("md") => "text/markdown",
                                _ => "application/octet-stream",
                            };
                            
                            // 다운로드 응답 생성
                            return HttpResponse::Ok()
                                .content_type(content_type)
                                .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
                                .body(file_bytes);
                        },
                        Err(e) => {
                            return HttpResponse::InternalServerError().json(json!({
                                "status": "error",
                                "message": format!("파일 읽기 실패: {}", e)
                            }));
                        }
                    }
                } else {
                    return HttpResponse::NotFound().json(json!({
                        "status": "error",
                        "message": "원본 파일을 찾을 수 없습니다."
                    }));
                }
            } else {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "작업 결과가 없습니다."
                }));
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
                "message": format!("작업 조회 중 오류가 발생했습니다: {}", e)
            }))
        }
    }
}

// GET /download/result/{job_id} - 파싱 결과 다운로드 (JSON 형식)
#[get("/download/result/{job_id}")]
pub async fn download_result(
    job_queue: web::Data<JobQueue>,
    job_id: web::Path<Uuid>
) -> impl Responder {
    // 작업 정보 조회
    match job_queue.get_job(*job_id).await {
        Ok(Some(job)) => {
            if let Some(result) = job.result {
                // 파일명 추출 (있는 경우)
                let filename = &result.filename;
                
                // 결과 JSON 직렬화
                match serde_json::to_string_pretty(&result) {
                    Ok(json_string) => {
                        // 다운로드 응답 생성
                        return HttpResponse::Ok()
                            .content_type("application/json")
                            .append_header(("Content-Disposition", format!("attachment; filename=\"{}_result.json\"", filename)))
                            .body(json_string);
                    },
                    Err(e) => {
                        return HttpResponse::InternalServerError().json(json!({
                            "status": "error",
                            "message": format!("JSON 직렬화 실패: {}", e)
                        }));
                    }
                }
            } else {
                return HttpResponse::BadRequest().json(json!({
                    "status": "error",
                    "message": "작업 결과가 없습니다."
                }));
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
                "message": format!("작업 조회 중 오류가 발생했습니다: {}", e)
            }))
        }
    }
}
