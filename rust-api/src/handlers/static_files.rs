use actix_web::{get, web, HttpResponse, Result as ActixResult};
use std::path::PathBuf;
use std::fs;

// 이미지 파일 서빙
#[get("/images/{filename}")]
pub async fn serve_image(path: web::Path<String>) -> ActixResult<HttpResponse> {
    let filename = path.into_inner();
    let file_path = PathBuf::from("./images").join(&filename);
    
    // 파일이 존재하는지 확인
    if !file_path.exists() {
        return Ok(HttpResponse::NotFound()
            .json(serde_json::json!({
                "error": "이미지 파일을 찾을 수 없습니다.",
                "filename": filename
            })));
    }
    
    // 파일 확장자 확인 (보안을 위해)
    let extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    match extension.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "webp" => {
            // 파일 읽기
            match fs::read(&file_path) {
                Ok(file_data) => {
                    // MIME 타입 결정
                    let content_type = match extension.to_lowercase().as_str() {
                        "png" => "image/png",
                        "jpg" | "jpeg" => "image/jpeg",
                        "gif" => "image/gif",
                        "webp" => "image/webp",
                        _ => "application/octet-stream"
                    };
                    
                    Ok(HttpResponse::Ok()
                        .content_type(content_type)
                        .body(file_data))
                },
                Err(_) => Ok(HttpResponse::InternalServerError()
                    .json(serde_json::json!({
                        "error": "파일을 읽을 수 없습니다.",
                        "filename": filename
                    })))
            }
        },
        _ => Ok(HttpResponse::BadRequest()
            .json(serde_json::json!({
                "error": "지원하지 않는 이미지 형식입니다.",
                "filename": filename,
                "allowed_types": ["png", "jpg", "jpeg", "gif", "webp"]
            })))
    }
}
