mod handlers;
mod models;
mod services;

use actix_web::{App, HttpServer, web};
use actix_cors::Cors;
use handlers::{health::healthz, upload::upload, job::{get_job_status, get_job_result}, download::{download_original, download_result}};
use services::job_queue::JobQueue;
use services::worker::BackgroundWorker;
use std::env;
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 로깅 초기화
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    println!("🚀 Document Parser System - Phase 4 시작");
    println!("📋 지원 포맷: PDF, DOCX, XLSX, TXT, MD");
    println!("🌐 서버 주소: http://0.0.0.0:8080");
    println!("⚙️ 비동기 작업 큐 활성화됨");
    
    // Redis URL 환경변수 또는 기본값 사용
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    info!("Redis 연결: {}", redis_url);
    
    let job_queue = JobQueue::new(&redis_url);
    
    // 백그라운드 워커 시작 (별도 스레드에서 실행)
    let worker_job_queue = job_queue.clone();
    tokio::spawn(async move {
        info!("백그라운드 워커 시작 중...");
        let worker = BackgroundWorker::new(worker_job_queue, 5); // 5초마다 작업 확인
        worker.start().await;
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        App::new()
            .app_data(web::Data::new(job_queue.clone()))
            .wrap(cors)
            .service(healthz)
            .service(upload)
            .service(get_job_status)
            .service(get_job_result)
            .service(download_original)
            .service(download_result)
    })
    .bind(("0.0.0.0", 8080))?
    .workers(2) // 워커 스레드 수 설정
    .run()
    .await
}
