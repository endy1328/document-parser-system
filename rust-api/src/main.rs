use actix_web::{get, post, App, HttpServer, Responder, HttpResponse, Error};
use actix_multipart::Multipart;
use futures_util::stream::StreamExt;
use std::io::Write;
use std::fs;
use serde_json::json;

#[get("/healthz")]
async fn healthz() -> impl Responder {
    "OK"
}

#[post("/upload")]
async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // uploads 폴더 생성
    let upload_dir = "uploads";
    fs::create_dir_all(upload_dir).ok();

    while let Some(item) = payload.next().await {
        let mut field = item?;
        // 필요한 정보를 먼저 복사
        let filename = field.content_disposition()
            .get_filename()
            .map(|s| s.to_string())
            .unwrap_or("file".to_string());
        let filepath = format!("{}/{}", upload_dir, filename);
        let is_pdf = filename.to_lowercase().ends_with(".pdf");

        let mut f = fs::File::create(&filepath)?;
        while let Some(chunk) = field.next().await {
            let chunk = chunk?;
            f.write_all(&chunk)?;
        }
        // PDF 텍스트 추출 (pdftotext 외부 명령어 사용)
        if is_pdf {
            use std::process::Command;
            match Command::new("pdftotext").arg(&filepath).arg("-").output() {
                Ok(output) => {
                    if output.status.success() {
                        let text = String::from_utf8_lossy(&output.stdout).to_string();
                        return Ok(HttpResponse::Ok().json(json!({
                            "status": "success",
                            "filename": filename,
                            "text": text
                        })));
                    } else {
                        let err = String::from_utf8_lossy(&output.stderr).to_string();
                        return Ok(HttpResponse::Ok().json(json!({
                            "status": "fail",
                            "filename": filename,
                            "error": format!("pdftotext 오류: {}", err)
                        })));
                    }
                },
                Err(e) => {
                    return Ok(HttpResponse::Ok().json(json!({
                        "status": "fail",
                        "filename": filename,
                        "error": format!("pdftotext 실행 실패: {}", e)
                    })));
                }
            }
        } else {
            return Ok(HttpResponse::Ok().json(json!({
                "status": "success",
                "filename": filename,
                "message": "PDF가 아닌 파일은 저장만 합니다."
            })));
        }
    }
    Ok(HttpResponse::BadRequest().body("파일이 업로드되지 않았습니다."))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(healthz)
            .service(upload)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
