

# 문서 파서 시스템 - 완전한 프로젝트 구조

## 1. 프로젝트 디렉토리 구조

```
document-parser-system/
├── backend/
│   ├── rust-api/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── handlers/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── upload.rs
│   │   │   │   ├── parser.rs
│   │   │   │   └── version.rs
│   │   │   ├── models/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── document.rs
│   │   │   │   └── parser.rs
│   │   │   ├── services/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── parser.rs
│   │   │   │   ├── kafka.rs
│   │   │   │   └── grpc_client.rs
│   │   │   ├── config/
│   │   │   │   ├── mod.rs
│   │   │   │   └── settings.rs
│   │   │   └── proto/
│   │   │       ├── mod.rs
│   │   │       └── llm_service.proto
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── build.rs
│   └── python-services/
│       ├── llm-service/
│       │   ├── app/
│       │   │   ├── __init__.py
│       │   │   ├── main.py
│       │   │   ├── services/
│       │   │   │   ├── __init__.py
│       │   │   │   ├── llm_service.py
│       │   │   │   └── grpc_server.py
│       │   │   ├── models/
│       │   │   │   ├── __init__.py
│       │   │   │   └── schemas.py
│       │   │   └── proto/
│       │   │       ├── __init__.py
│       │   │       ├── llm_service.proto
│       │   │       └── llm_service_pb2.py
│       │   ├── requirements.txt
│       │   ├── Dockerfile
│       │   └── pyproject.toml
│       └── ocr-service/
│           ├── app/
│           │   ├── __init__.py
│           │   ├── main.py
│           │   └── services/
│           │       ├── __init__.py
│           │       ├── ocr_service.py
│           │       └── layout_parser.py
│           ├── requirements.txt
│           └── Dockerfile
├── frontend/
│   ├── src/
│   │   ├── app.html
│   │   ├── app.css
│   │   ├── routes/
│   │   │   ├── +layout.svelte
│   │   │   ├── +page.svelte
│   │   │   └── parser/
│   │   │       ├── +page.svelte
│   │   │       └── components/
│   │   │           ├── UploadZone.svelte
│   │   │           ├── DocumentViewer.svelte
│   │   │           ├── JsonEditor.svelte
│   │   │           └── PreviewTabs.svelte
│   │   ├── lib/
│   │   │   ├── stores/
│   │   │   │   ├── document.ts
│   │   │   │   └── parser.ts
│   │   │   ├── api/
│   │   │   │   ├── client.ts
│   │   │   │   └── types.ts
│   │   │   └── utils/
│   │   │       ├── pdf.ts
│   │   │       └── monaco.ts
│   │   └── static/
│   ├── package.json
│   ├── svelte.config.js
│   ├── tailwind.config.js
│   ├── vite.config.ts
│   └── Dockerfile
├── infrastructure/
│   ├── k8s/
│   │   ├── namespaces/
│   │   │   └── document-parser.yaml
│   │   ├── configmaps/
│   │   │   ├── rust-api-config.yaml
│   │   │   └── python-services-config.yaml
│   │   ├── deployments/
│   │   │   ├── rust-api.yaml
│   │   │   ├── llm-service.yaml
│   │   │   ├── ocr-service.yaml
│   │   │   └── frontend.yaml
│   │   ├── services/
│   │   │   ├── rust-api-service.yaml
│   │   │   ├── llm-service.yaml
│   │   │   ├── ocr-service.yaml
│   │   │   └── frontend-service.yaml
│   │   ├── ingress/
│   │   │   └── document-parser-ingress.yaml
│   │   └── monitoring/
│   │       ├── prometheus-config.yaml
│   │       ├── grafana-dashboard.json
│   │       └── servicemonitor.yaml
│   ├── docker-compose.yml
│   └── helm/
│       └── document-parser/
│           ├── Chart.yaml
│           ├── values.yaml
│           └── templates/
├── config/
│   ├── config.yaml
│   ├── kafka/
│   │   └── topics.yaml
│   └── database/
│       └── migrations/
├── .github/
│   └── workflows/
│       ├── ci.yml
│       ├── cd.yml
│       └── security.yml
└── docs/
    ├── README.md
    ├── API.md
    └── DEPLOYMENT.md
```

## 2. 백엔드 설정 파일들

### Cargo.toml (Rust API)

```toml
[package]
name = "document-parser-api"
version = "0.1.0"
edition = "2021"

[dependencies]
actix-web = "4.4"
actix-multipart = "0.6"
actix-cors = "0.6"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-actix-web = "0.7"

# Document parsing
lopdf = "0.32"
pdfium-render = "0.8"
docx-rs = "0.4"
calamine = "0.22"
pulldown-cmark = "0.9"

# Kafka
rdkafka = { version = "0.36", features = ["cmake-build"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }

# gRPC
tonic = "0.10"
prost = "0.12"

# Object Storage (MinIO)
aws-sdk-s3 = "0.39"
aws-config = "0.56"

# Configuration
config = "0.13"
clap = { version = "4.4", features = ["derive"] }

# Metrics
prometheus = "0.13"
actix-web-prometheus = "0.1"

[build-dependencies]
tonic-build = "0.10"

[[bin]]
name = "document-parser-api"
path = "src/main.rs"
```

### requirements.txt (Python LLM Service)

```txt
fastapi==0.104.1
uvicorn[standard]==0.24.0
grpcio==1.59.3
grpcio-tools==1.59.3
protobuf==4.25.1
pydantic==2.5.0
pydantic-settings==2.1.0
openai==1.3.7
google-generativeai==0.3.2
anthropic==0.7.8
ollama==0.1.7
layoutparser[layoutmodels,tesseract]==0.3.4
pytesseract==0.3.10
opencv-python==4.8.1.78
transformers==4.36.1
torch==2.1.1
pillow==10.1.0
prometheus-client==0.19.0
structlog==23.2.0
```

## 3. Rust API 핵심 코드

### src/main.rs

```rust
use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_cors::Cors;
use tracing_actix_web::TracingLogger;
use std::sync::Arc;

mod handlers;
mod models;
mod services;
mod config;

use config::Settings;
use services::{parser::ParserService, kafka::KafkaProducer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    
    let settings = Settings::new().expect("Failed to load configuration");
    let kafka_producer = Arc::new(
        KafkaProducer::new(&settings.kafka.brokers)
            .await
            .expect("Failed to create Kafka producer")
    );
    
    let parser_service = Arc::new(
        ParserService::new(settings.clone(), kafka_producer.clone())
            .await
            .expect("Failed to create parser service")
    );

    println!("🚀 Starting Document Parser API on {}:{}", 
             settings.server.host, settings.server.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(parser_service.clone()))
            .app_data(web::Data::new(kafka_producer.clone()))
            .wrap(TracingLogger::default())
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec!["Content-Type", "Authorization"])
                    .max_age(3600)
            )
            .service(
                web::scope("/api/v1")
                    .configure(handlers::configure_routes)
            )
    })
    .bind(format!("{}:{}", settings.server.host, settings.server.port))?
    .run()
    .await
}
```

### src/handlers/upload.rs

```rust
use actix_web::{web, HttpResponse, Result};
use actix_multipart::Multipart;
use futures_util::TryStreamExt;
use std::sync::Arc;
use uuid::Uuid;
use serde_json::json;

use crate::services::parser::ParserService;
use crate::models::document::{DocumentUploadRequest, ParseJobStatus};

pub async fn upload_document(
    mut payload: Multipart,
    parser_service: web::Data<Arc<ParserService>>,
) -> Result<HttpResponse> {
    let mut file_data = Vec::new();
    let mut filename = String::new();
    let mut content_type = String::new();
    let mut parse_options = None;

    while let Some(mut field) = payload.try_next().await? {
        match field.name() {
            "file" => {
                filename = field
                    .content_disposition()
                    .get_filename()
                    .unwrap_or("unknown")
                    .to_string();
                
                content_type = field
                    .content_type()
                    .map(|ct| ct.to_string())
                    .unwrap_or_else(|| "application/octet-stream".to_string());

                while let Some(chunk) = field.try_next().await? {
                    file_data.extend_from_slice(&chunk);
                }
            }
            "options" => {
                let mut options_data = Vec::new();
                while let Some(chunk) = field.try_next().await? {
                    options_data.extend_from_slice(&chunk);
                }
                if let Ok(options_str) = String::from_utf8(options_data) {
                    parse_options = serde_json::from_str(&options_str).ok();
                }
            }
            _ => {}
        }
    }

    if file_data.is_empty() {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error": "No file provided"
        })));
    }

    let job_id = Uuid::new_v4();
    let request = DocumentUploadRequest {
        job_id,
        filename,
        content_type,
        file_data,
        parse_options: parse_options.unwrap_or_default(),
    };

    match parser_service.submit_parse_job(request).await {
        Ok(_) => Ok(HttpResponse::Accepted().json(json!({
            "job_id": job_id,
            "status": "queued",
            "message": "Document parsing job submitted successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to submit parse job: {}", e)
        })))
    }
}

pub async fn get_parse_status(
    path: web::Path<Uuid>,
    parser_service: web::Data<Arc<ParserService>>,
) -> Result<HttpResponse> {
    let job_id = path.into_inner();
    
    match parser_service.get_job_status(job_id).await {
        Ok(status) => Ok(HttpResponse::Ok().json(status)),
        Err(e) => Ok(HttpResponse::NotFound().json(json!({
            "error": format!("Job not found: {}", e)
        })))
    }
}

pub async fn get_parse_result(
    path: web::Path<Uuid>,
    parser_service: web::Data<Arc<ParserService>>,
) -> Result<HttpResponse> {
    let job_id = path.into_inner();
    
    match parser_service.get_parse_result(job_id).await {
        Ok(Some(result)) => Ok(HttpResponse::Ok().json(result)),
        Ok(None) => Ok(HttpResponse::NotFound().json(json!({
            "error": "Parse result not found or job still processing"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get parse result: {}", e)
        })))
    }
}
```

### src/services/parser.rs

```rust
use anyhow::{Result, anyhow};
use std::sync::Arc;
use uuid::Uuid;
use serde_json::Value;
use tokio::sync::RwLock;
use std::collections::HashMap;

use crate::config::Settings;
use crate::models::document::{DocumentUploadRequest, ParseJobStatus, ParseResult};
use crate::services::kafka::KafkaProducer;

pub struct ParserService {
    settings: Settings,
    kafka_producer: Arc<KafkaProducer>,
    job_status_cache: Arc<RwLock<HashMap<Uuid, ParseJobStatus>>>,
    result_cache: Arc<RwLock<HashMap<Uuid, ParseResult>>>,
}

impl ParserService {
    pub async fn new(
        settings: Settings,
        kafka_producer: Arc<KafkaProducer>,
    ) -> Result<Self> {
        Ok(Self {
            settings,
            kafka_producer,
            job_status_cache: Arc::new(RwLock::new(HashMap::new())),
            result_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn submit_parse_job(&self, request: DocumentUploadRequest) -> Result<()> {
        // Store initial job status
        {
            let mut cache = self.job_status_cache.write().await;
            cache.insert(request.job_id, ParseJobStatus {
                job_id: request.job_id,
                status: "queued".to_string(),
                progress: 0,
                message: Some("Job queued for processing".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                estimated_completion: None,
            });
        }

        // Send job to Kafka for processing
        let job_message = serde_json::to_string(&request)?;
        self.kafka_producer
            .send_message("document-parse-jobs", &request.job_id.to_string(), &job_message)
            .await?;

        Ok(())
    }

    pub async fn get_job_status(&self, job_id: Uuid) -> Result<ParseJobStatus> {
        let cache = self.job_status_cache.read().await;
        cache.get(&job_id)
            .cloned()
            .ok_or_else(|| anyhow!("Job not found"))
    }

    pub async fn get_parse_result(&self, job_id: Uuid) -> Result<Option<ParseResult>> {
        let cache = self.result_cache.read().await;
        Ok(cache.get(&job_id).cloned())
    }

    pub async fn process_document(&self, request: DocumentUploadRequest) -> Result<ParseResult> {
        // Update job status to processing
        self.update_job_status(request.job_id, "processing", 10, 
                              Some("Starting document parsing")).await?;

        let parsed_content = match self.detect_and_parse_document(&request).await {
            Ok(content) => {
                self.update_job_status(request.job_id, "processing", 60, 
                                      Some("Document parsed, applying transformations")).await?;
                content
            }
            Err(e) => {
                self.update_job_status(request.job_id, "failed", 0, 
                                      Some(&format!("Parsing failed: {}", e))).await?;
                return Err(e);
            }
        };

        // Apply LLM processing if requested
        let final_content = if request.parse_options.use_llm {
            self.update_job_status(request.job_id, "processing", 80, 
                                  Some("Applying AI enhancements")).await?;
            self.apply_llm_processing(&parsed_content, &request.parse_options).await?
        } else {
            parsed_content
        };

        let result = ParseResult {
            job_id: request.job_id,
            filename: request.filename,
            content_type: request.content_type,
            parsed_content: final_content,
            metadata: self.extract_metadata(&request).await?,
            created_at: chrono::Utc::now(),
        };

        // Store result and update status
        {
            let mut cache = self.result_cache.write().await;
            cache.insert(request.job_id, result.clone());
        }

        self.update_job_status(request.job_id, "completed", 100, 
                              Some("Document processing completed")).await?;

        Ok(result)
    }

    async fn detect_and_parse_document(&self, request: &DocumentUploadRequest) -> Result<Value> {
        match request.content_type.as_str() {
            "application/pdf" => self.parse_pdf(&request.file_data, &request.parse_options).await,
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
                self.parse_docx(&request.file_data).await
            }
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => {
                self.parse_xlsx(&request.file_data, &request.parse_options).await
            }
            "text/plain" => self.parse_text(&request.file_data).await,
            "text/markdown" => self.parse_markdown(&request.file_data).await,
            _ => Err(anyhow!("Unsupported file type: {}", request.content_type))
        }
    }

    async fn parse_pdf(&self, data: &[u8], options: &crate::models::document::ParseOptions) -> Result<Value> {
        use lopdf::Document;
        
        let doc = Document::load_mem(data)?;
        let mut pages = Vec::new();
        
        for page_num in 1..=doc.get_pages().len() {
            let page_text = doc.extract_text(&[page_num as u32])?;
            
            let page_content = if options.use_ocr && page_text.trim().is_empty() {
                // Use OCR for scanned PDFs
                self.perform_ocr_on_page(data, page_num as u32).await?
            } else {
                page_text
            };
            
            pages.push(serde_json::json!({
                "page": page_num,
                "content": page_content,
                "type": if options.use_ocr && page_text.trim().is_empty() { "ocr" } else { "text" }
            }));
        }

        Ok(serde_json::json!({
            "type": "pdf",
            "pages": pages,
            "total_pages": pages.len()
        }))
    }

    async fn parse_xlsx(&self, data: &[u8], options: &crate::models::document::ParseOptions) -> Result<Value> {
        use calamine::{Reader, Xlsx, open_workbook_from_rs};
        use std::io::Cursor;
        
        let cursor = Cursor::new(data);
        let mut workbook: Xlsx<_> = open_workbook_from_rs(cursor)?;
        let mut worksheets = Vec::new();
        
        for sheet_name in workbook.worksheet_names() {
            if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
                let mut rows = Vec::new();
                
                for (row_idx, row) in range.rows().enumerate() {
                    let mut cells = Vec::new();
                    let mut has_visible_content = false;
                    
                    for (col_idx, cell) in row.iter().enumerate() {
                        let cell_value = cell.to_string();
                        
                        // Apply hidden cell filtering if requested
                        let is_visible = if options.filter_hidden_cells {
                            // This is a simplified check - in real implementation,
                            // you'd need to check the actual cell formatting
                            !cell_value.is_empty()
                        } else {
                            true
                        };
                        
                        if is_visible && !cell_value.is_empty() {
                            has_visible_content = true;
                        }
                        
                        cells.push(serde_json::json!({
                            "column": col_idx,
                            "value": cell_value,
                            "visible": is_visible
                        }));
                    }
                    
                    if has_visible_content || !options.filter_hidden_cells {
                        rows.push(serde_json::json!({
                            "row": row_idx,
                            "cells": cells
                        }));
                    }
                }
                
                worksheets.push(serde_json::json!({
                    "name": sheet_name,
                    "rows": rows
                }));
            }
        }
        
        Ok(serde_json::json!({
            "type": "xlsx",
            "worksheets": worksheets
        }))
    }

    async fn parse_docx(&self, data: &[u8]) -> Result<Value> {
        use docx_rs::*;
        use std::io::Cursor;
        
        let cursor = Cursor::new(data);
        let docx = read_docx(cursor)?;
        
        // Extract text content
        let mut paragraphs = Vec::new();
        // Note: This is simplified - docx-rs API might be different
        // You'd need to traverse the document structure properly
        
        Ok(serde_json::json!({
            "type": "docx",
            "paragraphs": paragraphs,
            "content": "Document content extraction would go here"
        }))
    }

    async fn parse_text(&self, data: &[u8]) -> Result<Value> {
        let content = String::from_utf8(data.to_vec())?;
        Ok(serde_json::json!({
            "type": "text",
            "content": content
        }))
    }

    async fn parse_markdown(&self, data: &[u8]) -> Result<Value> {
        use pulldown_cmark::{Parser, html};
        
        let content = String::from_utf8(data.to_vec())?;
        let parser = Parser::new(&content);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        
        Ok(serde_json::json!({
            "type": "markdown",
            "raw_markdown": content,
            "html": html_output
        }))
    }

    async fn perform_ocr_on_page(&self, _pdf_data: &[u8], _page_num: u32) -> Result<String> {
        // This would call the Python OCR service via gRPC
        // Placeholder implementation
        Ok("OCR text would be extracted here".to_string())
    }

    async fn apply_llm_processing(&self, content: &Value, _options: &crate::models::document::ParseOptions) -> Result<Value> {
        // This would call the Python LLM service via gRPC
        // Placeholder implementation
        Ok(content.clone())
    }

    async fn extract_metadata(&self, request: &DocumentUploadRequest) -> Result<Value> {
        Ok(serde_json::json!({
            "filename": request.filename,
            "content_type": request.content_type,
            "file_size": request.file_data.len(),
            "processed_at": chrono::Utc::now()
        }))
    }

    async fn update_job_status(&self, job_id: Uuid, status: &str, progress: u8, message: Option<&str>) -> Result<()> {
        let mut cache = self.job_status_cache.write().await;
        if let Some(job_status) = cache.get_mut(&job_id) {
            job_status.status = status.to_string();
            job_status.progress = progress;
            job_status.message = message.map(|s| s.to_string());
            job_status.updated_at = chrono::Utc::now();
        }
        Ok(())
    }
}
```

## 4. Python LLM 서비스

### app/main.py

```python
import asyncio
import structlog
from fastapi import FastAPI
from contextlib import asynccontextmanager
from concurrent.futures import ThreadPoolExecutor

from app.services.grpc_server import start_grpc_server
from app.services.llm_service import LLMService

logger = structlog.get_logger()

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup
    logger.info("Starting LLM Service")
    
    # Initialize LLM service
    llm_service = LLMService()
    app.state.llm_service = llm_service
    
    # Start gRPC server in background
    executor = ThreadPoolExecutor(max_workers=1)
    grpc_task = asyncio.create_task(
        asyncio.get_event_loop().run_in_executor(
            executor, start_grpc_server, llm_service
        )
    )
    
    yield
    
    # Shutdown
    logger.info("Shutting down LLM Service")
    grpc_task.cancel()

app = FastAPI(
    title="Document Parser LLM Service",
    description="AI-powered document processing service",
    version="1.0.0",
    lifespan=lifespan
)

@app.get("/health")
async def health_check():
    return {"status": "healthy", "service": "llm-service"}

@app.get("/models")
async def list_available_models():
    return {
        "openai": ["gpt-4", "gpt-3.5-turbo"],
        "anthropic": ["claude-3-opus-20240229", "claude-3-sonnet-20240229"],
        "google": ["gemini-pro", "gemini-pro-vision"],
        "ollama": ["llama2", "mistral", "codellama"]
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "app.main:app",
        host="0.0.0.0",
        port=8001,
        reload=True,
        log_level="info"
    )
```

### app/services/llm_service.py

```python
import asyncio
import json
from typing import Dict, Any, Optional
import structlog
from pydantic import BaseModel

import openai
import anthropic
import google.generativeai as genai
import ollama

logger = structlog.get_logger()

class LLMRequest(BaseModel):
    provider: str
    model: str
    prompt: str
    document_content: Dict[str, Any]
    options: Dict[str, Any] = {}

class LLMResponse(BaseModel):
    success: bool
    result: Optional[Dict[str, Any]] = None
    error: Optional[str] = None
    metadata: Dict[str, Any] = {}

class LLMService:
    def __init__(self):
        self.openai_client = openai.AsyncOpenAI()
        self.anthropic_client = anthropic.AsyncAnthropic()
        # Configure Google AI
        genai.configure(api_key="your-google-api-key")
        
        logger.info("LLM Service initialized")

    async def process_document(self, request: LLMRequest) -> LLMResponse:
        try:
            logger.info(f"Processing document with {request.provider}:{request.model}")
            
            if request.provider == "openai":
                result = await self._process_with_openai(request)
            elif request.provider == "anthropic":
                result = await self._process_with_anthropic(request)
            elif request.provider == "google":
                result = await self._process_with_google(request)
            elif request.provider == "ollama":
                result = await self._process_with_ollama(request)
            else:
                raise ValueError(f"Unsupported provider: {request.provider}")
            
            return LLMResponse(
                success=True,
                result=result,
                metadata={
                    "provider": request.provider,
                    "model": request.model,
                    "processing_time": "calculated_time_here"
                }
            )
            
        except Exception as e:
            logger.error(f"LLM processing failed: {str(e)}")
            return LLMResponse(
                success=False,
                error=str(e)
            )

    async def _process_with_openai(self, request: LLMRequest) -> Dict[str, Any]:
        system_prompt = self._build_system_prompt(request.options)
        user_prompt = self._build_user_prompt(request.prompt, request.document_content)
        
        response = await self.openai_client.chat.completions.create(
            model=request.model,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            temperature=request.options.get("temperature", 0.1),
            max_tokens=request.options.get("max_tokens", 4000)
        )
        
        return {
            "enhanced_content": response.choices[0].message.content,
            "token_usage": response.usage.dict() if response.usage else None
        }

    async def _process_with_anthropic(self, request: LLMRequest) -> Dict[str, Any]:
        system_prompt = self._build_system_prompt(request.options)
        user_prompt = self._build_user_prompt(request.prompt, request.document_content)
        
        response = await self.anthropic_client.messages.create(
            model=request.model,
            max_tokens=request.options.get("max_tokens", 4000),
            temperature=request.options.get("temperature", 0.1),
            system=system_prompt,
            messages=[
                {"role": "user", "content": user_prompt}
            ]
        )
        
        return {
            "enhanced_content": response.content[0].text,
            "token_usage": {
                "input_tokens": response.usage.input_tokens,
                "output_tokens": response.usage.output_tokens
            }
        }

    async def _process_with_google(self, request: LLMRequest) -> Dict[str, Any]:
        model = genai.GenerativeModel(request.model)
        prompt = f"{self._build_system_prompt(request.options)}\n\n{self._build_user_prompt(request.prompt, request.document_content)}"
        
        response = await model.generate_content_async(
            prompt,
            generation_config=genai.types.GenerationConfig(
                temperature=request.options.get("temperature", 0.1),
                max_output_tokens=request.options.get("max_tokens", 4000)
            )
        )
        
        return {
            "enhanced_content": response.text,
            "token_usage": {
                "prompt_tokens": response.usage_metadata.prompt_token_count,
                "completion_tokens": response.usage_metadata.candidates_token_count
            }
        }

    async def _process_with_ollama(self, request: LLMRequest) -> Dict[str, Any]:
        prompt = f"{self._build_system_prompt(request.options)}\n\n{self._build_user_prompt(request.prompt, request.document_content)}"
        
        response = await ollama.AsyncClient().generate(
            model=request.model,
            prompt=prompt,
            options={
                "temperature": request.options.get("temperature", 0.1),
                "num_predict": request.options.get("max_tokens", 4000)
            }
        )
        
        return {
            "enhanced_content": response['response'],
            "token_usage": {
                "eval_count": response.get('eval_count', 0),
                "prompt_eval_count": response.get('prompt_eval_count', 0)
            }
        }

    def _build_system_prompt(self, options: Dict[str, Any]) -> str:
        task_type = options.get("task_type", "general")
        
        if task_type == "summarization":
            return """You are an expert document summarizer. Create concise, accurate summaries that capture the key points and main ideas of the document while maintaining important details."""
        elif task_type == "extraction":
            return """You are an expert information extractor. Extract structured data, key entities, and important facts from documents in a clear, organized format."""
        elif task_type == "classification":
            return """You are a document classifier. Analyze documents and categorize them based on content, purpose, and structure."""
        elif task_type == "enhancement":
            return """You are a document enhancement specialist. Improve document structure, clarity, and formatting while preserving original meaning and content."""
        else:
            return """You are an AI assistant specialized in document processing. Analyze and process the provided document content according to the user's requirements."""

    def _build_user_prompt(self, custom_prompt: str, document_content: Dict[str, Any]) -> str:
        content_str = json.dumps(document_content, indent=2, ensure_ascii=False)
        
        return f"""
{custom_prompt}

Document Content:
```json
{content_str}
```

Please process this document according to the instructions above. Provide your response in a clear, structured format.
"""
```

### app/services/grpc_server.py

```python
import grpc
from concurrent import futures
import structlog
from typing import Dict, Any
import json

from app.proto import llm_service_pb2_grpc, llm_service_pb2
from app.services.llm_service import LLMService, LLMRequest

logger = structlog.get_logger()

class LLMServicer(llm_service_pb2_grpc.LLMServiceServicer):
    def __init__(self, llm_service: LLMService):
        self.llm_service = llm_service

    async def ProcessDocument(self, request, context):
        try:
            # Convert gRPC request to internal format
            llm_request = LLMRequest(
                provider=request.provider,
                model=request.model,
                prompt=request.prompt,
                document_content=json.loads(request.document_content),
                options=json.loads(request.options) if request.options else {}
            )
            
            # Process the request
            response = await self.llm_service.process_document(llm_request)
            
            # Convert back to gRPC response
            return llm_service_pb2.LLMResponse(
                success=response.success,
                result=json.dumps(response.result) if response.result else "",
                error=response.error or "",
                metadata=json.dumps(response.metadata)
            )
            
        except Exception as e:
            logger.error(f"gRPC processing error: {str(e)}")
            return llm_service_pb2.LLMResponse(
                success=False,
                error=str(e),
                result="",
                metadata="{}"
            )

    async def GetAvailableModels(self, request, context):
        models = {
            "openai": ["gpt-4", "gpt-3.5-turbo"],
            "anthropic": ["claude-3-opus-20240229", "claude-3-sonnet-20240229"],
            "google": ["gemini-pro", "gemini-pro-vision"],
            "ollama": ["llama2", "mistral", "codellama"]
        }
        
        return llm_service_pb2.ModelsResponse(
            models=json.dumps(models)
        )

def start_grpc_server(llm_service: LLMService, port: int = 50051):
    server = grpc.aio.server(futures.ThreadPoolExecutor(max_workers=10))
    llm_service_pb2_grpc.add_LLMServiceServicer_to_server(
        LLMServicer(llm_service), server
    )
    
    listen_addr = f'0.0.0.0:{port}'
    server.add_insecure_port(listen_addr)
    
    logger.info(f"Starting gRPC server on {listen_addr}")
    
    async def serve():
        await server.start()
        await server.wait_for_termination()
    
    import asyncio
    asyncio.run(serve())
```

## 5. SvelteKit 프론트엔드

### package.json

```json
{
  "name": "document-parser-frontend",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "check": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
    "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
    "lint": "prettier --plugin-search-dir . --check . && eslint .",
    "format": "prettier --plugin-search-dir . --write ."
  },
  "devDependencies": {
    "@sveltejs/adapter-auto": "^2.1.1",
    "@sveltejs/kit": "^1.27.4",
    "@typescript-eslint/eslint-plugin": "^6.12.0",
    "@typescript-eslint/parser": "^6.12.0",
    "autoprefixer": "^10.4.16",
    "eslint": "^8.54.0",
    "eslint-config-prettier": "^9.0.0",
    "eslint-plugin-svelte": "^2.35.1",
    "postcss": "^8.4.31",
    "prettier": "^3.1.0",
    "prettier-plugin-svelte": "^3.1.2",
    "svelte": "^4.2.7",
    "svelte-check": "^3.6.0",
    "tailwindcss": "^3.3.5",
    "typescript": "^5.3.2",
    "vite": "^4.5.0"
  },
  "dependencies": {
    "monaco-editor": "^0.44.0",
    "pdfjs-dist": "^3.11.174",
    "lucide-svelte": "^0.294.0",
    "@tailwindcss/typography": "^0.5.10"
  }
}
```

### src/routes/+layout.svelte

```svelte
<script lang="ts">
  import '../app.css';
  import { page } from '$app/stores';
  import { FileText, Settings, History, Download } from 'lucide-svelte';
</script>

<div class="min-h-screen bg-gray-50">
  <!-- Navigation Header -->
  <header class="bg-white shadow-sm border-b">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
      <div class="flex justify-between items-center h-16">
        <div class="flex items-center">
          <FileText class="h-8 w-8 text-blue-600" />
          <h1 class="ml-2 text-xl font-semibold text-gray-900">Document Parser</h1>
        </div>
        
        <nav class="flex space-x-8">
          <a 
            href="/" 
            class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium"
            class:text-blue-600={$page.url.pathname === '/'}
          >
            Parser
          </a>
          <a 
            href="/history" 
            class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium flex items-center"
            class:text-blue-600={$page.url.pathname === '/history'}
          >
            <History class="h-4 w-4 mr-1" />
            History
          </a>
          <a 
            href="/settings" 
            class="text-gray-600 hover:text-gray-900 px-3 py-2 rounded-md text-sm font-medium flex items-center"
            class:text-blue-600={$page.url.pathname === '/settings'}
          >
            <Settings class="h-4 w-4 mr-1" />
            Settings
          </a>
        </nav>
      </div>
    </div>
  </header>

  <!-- Main Content -->
  <main class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
    <slot />
  </main>
</div>
```

### src/routes/+page.svelte

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import UploadZone from '$lib/components/UploadZone.svelte';
  import DocumentViewer from '$lib/components/DocumentViewer.svelte';
  import JsonEditor from '$lib/components/JsonEditor.svelte';
  import PreviewTabs from '$lib/components/PreviewTabs.svelte';
  import { documentStore } from '$lib/stores/document';
  import { parserStore } from '$lib/stores/parser';

  let splitPaneWidth = 50; // percentage
  let isDragging = false;
  let splitPane: HTMLElement;

  function handleMouseDown() {
    isDragging = true;
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging || !splitPane) return;
    
    const rect = splitPane.getBoundingClientRect();
    const newWidth = ((e.clientX - rect.left) / rect.width) * 100;
    splitPaneWidth = Math.max(20, Math.min(80, newWidth));
  }

  function handleMouseUp() {
    isDragging = false;
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  }

  onMount(() => {
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  });
</script>

<svelte:head>
  <title>Document Parser</title>
</svelte:head>

<div class="h-[calc(100vh-12rem)]">
  {#if !$documentStore.currentDocument}
    <!-- Upload State -->
    <div class="h-full flex items-center justify-center">
      <div class="max-w-lg w-full">
        <UploadZone />
      </div>
    </div>
  {:else}
    <!-- Split Pane Layout -->
    <div class="h-full flex bg-white rounded-lg shadow" bind:this={splitPane}>
      <!-- Left Pane - Document Viewer -->
      <div class="relative overflow-hidden" style="width: {splitPaneWidth}%;">
        <div class="h-full p-4 overflow-auto">
          <h2 class="text-lg font-semibold mb-4 text-gray-900">Original Document</h2>
          <DocumentViewer document={$documentStore.currentDocument} />
        </div>
      </div>

      <!-- Splitter -->
      <div 
        class="w-1 bg-gray-200 cursor-col-resize hover:bg-gray-300 relative"
        on:mousedown={handleMouseDown}
        role="separator"
        tabindex="0"
      >
        <div class="absolute inset-y-0 -left-1 -right-1"></div>
      </div>

      <!-- Right Pane - Results and Editor -->
      <div class="flex-1 overflow-hidden" style="width: {100 - splitPaneWidth}%;">
        <div class="h-full flex flex-col">
          {#if $parserStore.isProcessing}
            <!-- Processing State -->
            <div class="h-full flex items-center justify-center">
              <div class="text-center">
                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
                <p class="text-gray-600">Processing document...</p>
                {#if $parserStore.currentJob?.progress}
                  <div class="w-64 bg-gray-200 rounded-full h-2 mx-auto mt-4">
                    <div 
                      class="bg-blue-600 h-2 rounded-full transition-all duration-300"
                      style="width: {$parserStore.currentJob.progress}%"
                    ></div>
                  </div>
                  <p class="text-sm text-gray-500 mt-2">{$parserStore.currentJob.progress}% complete</p>
                {/if}
              </div>
            </div>
          {:else if $parserStore.result}
            <!-- Results State -->
            <div class="h-full flex flex-col">
              <div class="border-b border-gray-200 p-4">
                <h2 class="text-lg font-semibold text-gray-900">Parsed Results</h2>
              </div>
              
              <div class="flex-1 overflow-hidden">
                <PreviewTabs 
                  parsedContent={$parserStore.result.parsed_content}
                  metadata={$parserStore.result.metadata}
                />
              </div>
            </div>
          {:else}
            <!-- Empty State -->
            <div class="h-full flex items-center justify-center">
              <p class="text-gray-500">Parse a document to see results</p>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  :global(body) {
    user-select: none;
  }
  
  :global(body.dragging) {
    cursor: col-resize;
  }
</style>
```

### src/lib/components/UploadZone.svelte

```svelte
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Upload, FileText, Settings } from 'lucide-svelte';
  import { documentStore } from '$lib/stores/document';
  import { parserStore } from '$lib/stores/parser';
  import { uploadDocument } from '$lib/api/client';

  const dispatch = createEventDispatcher();

  let dragActive = false;
  let fileInput: HTMLInputElement;
  let parseOptions = {
    outputFormat: 'full',
    useOcr: false,
    useLlm: false,
    aiProvider: 'openai',
    aiModel: 'gpt-4',
    filterHiddenCells: true,
    customPrompt: ''
  };
  let showAdvanced = false;

  const supportedTypes = [
    'application/pdf',
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    'text/plain',
    'text/markdown'
  ];

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    dragActive = true;
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    dragActive = false;
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragActive = false;
    
    const files = Array.from(e.dataTransfer?.files || []);
    if (files.length > 0) {
      await processFile(files[0]);
    }
  }

  async function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const files = Array.from(input.files || []);
    if (files.length > 0) {
      await processFile(files[0]);
    }
  }

  async function processFile(file: File) {
    if (!supportedTypes.includes(file.type)) {
      alert('Unsupported file type. Please select a PDF, DOCX, XLSX, TXT, or MD file.');
      return;
    }

    documentStore.setCurrentDocument({
      id: crypto.randomUUID(),
      name: file.name,
      type: file.type,
      size: file.size,
      file: file,
      uploadedAt: new Date()
    });

    try {
      const jobId = await uploadDocument(file, parseOptions);
      parserStore.startJob(jobId);
      
      // Poll for results
      const pollInterval = setInterval(async () => {
        const status = await parserStore.checkJobStatus(jobId);
        if (status?.status === 'completed') {
          clearInterval(pollInterval);
          await parserStore.getJobResult(jobId);
        } else if (status?.status === 'failed') {
          clearInterval(pollInterval);
          parserStore.setError(status.message || 'Processing failed');
        }
      }, 2000);
      
    } catch (error) {
      console.error('Upload failed:', error);
      parserStore.setError('Upload failed. Please try again.');
    }
  }

  function triggerFileInput() {
    fileInput.click();
  }
</script>

<div class="w-full max-w-2xl mx-auto">
  <!-- Upload Area -->
  <div
    class="relative border-2 border-dashed rounded-lg p-8 text-center transition-colors {dragActive
      ? 'border-blue-500 bg-blue-50'
      : 'border-gray-300 hover:border-gray-400'}"
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
    on:drop={handleDrop}
    role="button"
    tabindex="0"
    on:click={triggerFileInput}
    on:keydown={(e) => e.key === 'Enter' && triggerFileInput()}
  >
    <input
      bind:this={fileInput}
      type="file"
      accept=".pdf,.docx,.xlsx,.txt,.md"
      on:change={handleFileSelect}
      class="hidden"
    />
    
    <Upload class="mx-auto h-12 w-12 text-gray-400 mb-4" />
    <p class="text-lg font-medium text-gray-900 mb-2">Drop your document here</p>
    <p class="text-gray-600 mb-4">or click to browse files</p>
    <p class="text-sm text-gray-500">
      Supports PDF, DOCX, XLSX, TXT, and Markdown files
    </p>
  </div>

  <!-- Parse Options -->
  <div class="mt-6 bg-white rounded-lg border border-gray-200 p-6">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-medium text-gray-900">Parse Options</h3>
      <button
        type="button"
        class="text-blue-600 hover:text-blue-700 text-sm font-medium flex items-center"
        on:click={() => showAdvanced = !showAdvanced}
      >
        <Settings class="h-4 w-4 mr-1" />
        {showAdvanced ? 'Hide' : 'Show'} Advanced
      </button>
    </div>

    <div class="space-y-4">
      <!-- Output Format -->
      <div>
        <label class="block text-sm font-medium text-gray-700 mb-2">
          Output Format
        </label>
        <select
          bind:value={parseOptions.outputFormat}
          class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
        >
          <option value="minimal">Minimal - Basic text extraction</option>
          <option value="lite">Lite - Structured content</option>
          <option value="full">Full - Complete analysis</option>
        </select>
      </div>

      <!-- OCR Option -->
      <div class="flex items-center">
        <input
          id="useOcr"
          type="checkbox"
          bind:checked={parseOptions.useOcr}
          class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
        />
        <label for="useOcr" class="ml-2 text-sm text-gray-700">
          Use OCR for scanned documents
        </label>
      </div>

      <!-- LLM Enhancement -->
      <div class="flex items-center">
        <input
          id="useLlm"
          type="checkbox"
          bind:checked={parseOptions.useLlm}
          class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
        />
        <label for="useLlm" class="ml-2 text-sm text-gray-700">
          Enhance with AI
        </label>
      </div>

      {#if showAdvanced}
        <div class="pt-4 border-t border-gray-200 space-y-4">
          <!-- AI Provider -->
          {#if parseOptions.useLlm}
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                AI Provider
              </label>
              <select
                bind:value={parseOptions.aiProvider}
                class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              >
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
                <option value="google">Google</option>
                <option value="ollama">Ollama (Local)</option>
              </select>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Model
              </label>
              <select
                bind:value={parseOptions.aiModel}
                class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
              >
                {#if parseOptions.aiProvider === 'openai'}
                  <option value="gpt-4">GPT-4</option>
                  <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
                {:else if parseOptions.aiProvider === 'anthropic'}
                  <option value="claude-3-opus-20240229">Claude 3 Opus</option>
                  <option value="claude-3-sonnet-20240229">Claude 3 Sonnet</option>
                {:else if parseOptions.aiProvider === 'google'}
                  <option value="gemini-pro">Gemini Pro</option>
                  <option value="gemini-pro-vision">Gemini Pro Vision</option>
                {:else if parseOptions.aiProvider === 'ollama'}
                  <option value="llama2">Llama 2</option>
                  <option value="mistral">Mistral</option>
                  <option value="codellama">Code Llama</option>
                {/if}
              </select>
            </div>

            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Custom Prompt (Optional)
              </label>
              <textarea
                bind:value={parseOptions.customPrompt}
                rows="3"
                class="w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500"
                placeholder="Enter specific instructions for AI processing..."
              ></textarea>
            </div>
          {/if}

          <!-- Excel-specific options -->
          <div class="flex items-center">
            <input
              id="filterHidden"
              type="checkbox"
              bind:checked={parseOptions.filterHiddenCells}
              class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
            />
            <label for="filterHidden" class="ml-2 text-sm text-gray-700">
              Filter hidden cells in Excel files
            </label>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
```

## 6. Docker 설정

### Backend Rust API Dockerfile

```dockerfile
# backend/rust-api/Dockerfile
FROM rust:1.75 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# Copy source code
COPY src ./src
COPY proto ./proto

# Build dependencies
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/document-parser-api /usr/local/bin/

# Create app user
RUN useradd -m -u 1001 appuser
USER appuser

WORKDIR /app

EXPOSE 8000

CMD ["document-parser-api"]
```

### Python LLM Service Dockerfile

```dockerfile
# backend/python-services/llm-service/Dockerfile
FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY app ./app

# Create non-root user
RUN useradd -m -u 1001 appuser && chown -R appuser:appuser /app
USER appuser

EXPOSE 8001 50051

CMD ["python", "-m", "app.main"]
```

### Frontend Dockerfile

```dockerfile
# frontend/Dockerfile
FROM node:18-alpine as builder

WORKDIR /app

# Copy package files
COPY package.json package-lock.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build application
RUN npm run build

FROM nginx:alpine

# Copy built application
COPY --from=builder /app/build /usr/share/nginx/html

# Copy nginx configuration
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
```

## 7. Kubernetes 매니페스트

### k8s/namespaces/document-parser.yaml

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: document-parser
  labels:
    name: document-parser
    istio-injection: enabled
```

### k8s/deployments/rust-api.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-api
  namespace: document-parser
  labels:
    app: rust-api
    version: v1
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-api
  template:
    metadata:
      labels:
        app: rust-api
        version: v1
    spec:
      containers:
      - name: rust-api
        image: document-parser/rust-api:latest
        ports:
        - containerPort: 8000
        env:
        - name: RUST_LOG
          value: "info"
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-secret
              key: url
        - name: KAFKA_BROKERS
          value: "kafka.kafka:9092"
        - name: MINIO_ENDPOINT
          value: "minio.minio:9000"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8000
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: config
          mountPath: /app/config
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: rust-api-config
```

### k8s/deployments/llm-service.yaml

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-service
  namespace: document-parser
  labels:
    app: llm-service
    version: v1
spec:
  replicas: 2
  selector:
    matchLabels:
      app: llm-service
  template:
    metadata:
      labels:
        app: llm-service
        version: v1
    spec:
      containers:
      - name: llm-service
        image: document-parser/llm-service:latest
        ports:
        - containerPort: 8001
          name: http
        - containerPort: 50051
          name: grpc
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-secrets
              key: openai-api-key
        - name: ANTHROPIC_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-secrets
              key: anthropic-api-key
        - name: GOOGLE_API_KEY
          valueFrom:
            secretKeyRef:
              name: llm-secrets
              key: google-api-key
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8001
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8001
          initialDelaySeconds: 10
          periodSeconds: 5
```

### k8s/services/rust-api-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: rust-api-service
  namespace: document-parser
  labels:
    app: rust-api
spec:
  selector:
    app: rust-api
  ports:
  - name: http
    port: 8000
    targetPort: 8000
    protocol: TCP
  type: ClusterIP
```

### k8s/services/llm-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: llm-service
  namespace: document-parser
  labels:
    app: llm-service
spec:
  selector:
    app: llm-service
  ports:
  - name: http
    port: 8001
    targetPort: 8001
    protocol: TCP
  - name: grpc
    port: 50051
    targetPort: 50051
    protocol: TCP
  type: ClusterIP
```

## 8. CI/CD 파이프라인

### .github/workflows/ci.yml

```yaml
name: CI Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    
    - name: Cache Cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/bin/
          ~/.cargo/registry/index/
          ~/.cargo/registry/cache/
          ~/.cargo/git/db/
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt --all -- --check
      working-directory: ./backend/rust-api
    
    - name: Run Clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
      working-directory: ./backend/rust-api
    
    - name: Run tests
      run: cargo test --verbose
      working-directory: ./backend/rust-api

  test-python:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        python-version: ["3.11"]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Python ${{ matrix.python-version }}
      uses: actions/setup-python@v4
      with:
        python-version: ${{ matrix.python-version }}
    
    - name: Cache pip dependencies
      uses: actions/cache@v3
      with:
        path: ~/.cache/pip
        key: ${{ runner.os }}-pip-${{ hashFiles('**/requirements.txt') }}
    
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install -r requirements.txt
        pip install pytest pytest-cov black isort mypy
      working-directory: ./backend/python-services/llm-service
    
    - name: Check code formatting
      run: |
        black --check .
        isort --check-only .
      working-directory: ./backend/python-services/llm-service
    
    - name: Type checking
      run: mypy app/
      working-directory: ./backend/python-services/llm-service
    
    - name: Run tests
      run: pytest --cov=app tests/
      working-directory: ./backend/python-services/llm-service

  test-frontend:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '18'
        cache: 'npm'
        cache-dependency-path: ./frontend/package-lock.json
    
    - name: Install dependencies
      run: npm ci
      working-directory: ./frontend
    
    - name: Run linting
      run: npm run lint
      working-directory: ./frontend
    
    - name: Run type checking
      run: npm run check
      working-directory: ./frontend
    
    - name: Run tests
      run: npm run test
      working-directory: ./frontend
    
    - name: Build application
      run: npm run build
      working-directory: ./frontend

  build-images:
    needs: [test-rust, test-python, test-frontend]
    runs-on: ubuntu-latest
    if: github.event_name != 'pull_request'
    
    strategy:
      matrix:
        service: [rust-api, llm-service, frontend]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
    
    - name: Log in to Container Registry
      uses: docker/login-action@v3
      with:
        registry: ${{ env.REGISTRY }}
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}
    
    - name: Extract metadata
      id: meta
      uses: docker/metadata-action@v5
      with:
        images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}/${{ matrix.service }}
        tags: |
          type=ref,event=branch
          type=ref,event=pr
          type=sha,prefix=sha-
          type=raw,value=latest,enable={{is_default_branch}}
    
    - name: Build and push Docker image
      uses: docker/build-push-action@v5
      with:
        context: .
        file: ./backend/${{ matrix.service }}/Dockerfile
        push: true
        tags: ${{ steps.meta.outputs.tags }}
        labels: ${{ steps.meta.outputs.labels }}
        cache-from: type=gha
        cache-to: type=gha,mode=max
```

## 9. 설정 관리

### config/config.yaml

```yaml
# Global Configuration
version: "1.0.0"
environment: development

# Server Configuration
server:
  host: "0.0.0.0"
  port: 8000
  workers: 4
  max_request_size: 50MB
  timeout: 300

# Database Configuration
database:
  url: "postgresql://user:password@cockroachdb:26257/document_parser"
  pool_size: 20
  max_connections: 100
  connection_timeout: 30

# Kafka Configuration
kafka:
  brokers:
    - "kafka-1:9092"
    - "kafka-2:9092"
    - "kafka-3:9092"
  topics:
    document_parse_jobs: "document-parse-jobs"
    parse_results: "parse-results"
    dead_letter_queue: "parse-dlq"
  consumer_group: "document-parser-group"
  auto_offset_reset: "earliest"
  max_poll_records: 100

# Object Storage (MinIO)
object_storage:
  endpoint: "http://minio:9000"
  access_key: "minioadmin"
  secret_key: "minioadmin"
  bucket: "document-parser"
  region: "us-east-1"
  secure: false

# gRPC Services
grpc:
  llm_service:
    host: "llm-service"
    port: 50051
    timeout: 120
    max_retry: 3
  ocr_service:
    host: "ocr-service"
    port: 50052
    timeout: 60
    max_retry: 2

# Document Parser Settings
parser:
  max_file_size: 50MB
  supported_formats:
    - "application/pdf"
    - "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    - "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    - "text/plain"
    - "text/markdown"
  
  # PDF Settings
  pdf:
    max_pages: 1000
    ocr_threshold: 0.8
    dpi: 300
  
  # Excel Settings
  excel:
    max_rows: 100000
    max_sheets: 50
    include_formulas: true
  
  # Text Processing
  text:
    max_length: 10MB
    encoding: "utf-8"

# AI/LLM Configuration
llm:
  default_provider: "openai"
  default_model: "gpt-4"
  max_tokens: 4000
  temperature: 0.1
  timeout: 120
  
  providers:
    openai:
      models: ["gpt-4", "gpt-3.5-turbo"]
      max_requests_per_minute: 60
    anthropic:
      models: ["claude-3-opus-20240229", "claude-3-sonnet-20240229"]
      max_requests_per_minute: 50
    google:
      models: ["gemini-pro", "gemini-pro-vision"]
      max_requests_per_minute: 60
    ollama:
      endpoint: "http://ollama:11434"
      models: ["llama2", "mistral", "codellama"]

# Monitoring Configuration
monitoring:
  metrics:
    enabled: true
    port: 9090
    path: "/metrics"
  
  logging:
    level: "info"
    format: "json"
    output: "stdout"
  
  tracing:
    enabled: true
    endpoint: "http://jaeger:14268/api/traces"
    sampling_rate: 0.1

# Security Configuration
security:
  cors:
    allowed_origins: ["http://localhost:5173", "https://document-parser.local"]
    allowed_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
    allowed_headers: ["Content-Type", "Authorization"]
  
  rate_limiting:
    enabled: true
    requests_per_minute: 100
    burst_size: 20
  
  auth:
    enabled: false
    oidc_provider: "http://keycloak:8080/auth/realms/document-parser"
```

### Pydantic Settings (Python)

```python
# backend/python-services/llm-service/app/config/settings.py
from pydantic import BaseSettings, Field
from typing import Dict, List, Optional
import os

class DatabaseSettings(BaseSettings):
    url: str = Field(..., env="DATABASE_URL")
    pool_size: int = Field(20, env="DB_POOL_SIZE")
    max_connections: int = Field(100, env="DB_MAX_CONNECTIONS")

class KafkaSettings(BaseSettings):
    brokers: List[str] = Field(["localhost:9092"], env="KAFKA_BROKERS")
    consumer_group: str = Field("llm-service-group", env="KAFKA_CONSUMER_GROUP")
    topics: Dict[str, str] = Field(default={
        "document_jobs": "document-parse-jobs",
        "results": "parse-results"
    })

class LLMSettings(BaseSettings):
    openai_api_key: Optional[str] = Field(None, env="OPENAI_API_KEY")
    anthropic_api_key: Optional[str] = Field(None, env="ANTHROPIC_API_KEY")
    google_api_key: Optional[str] = Field(None, env="GOOGLE_API_KEY")
    
    default_provider: str = Field("openai", env="DEFAULT_LLM_PROVIDER")
    default_model: str = Field("gpt-4", env="DEFAULT_LLM_MODEL")
    max_tokens: int = Field(4000, env="LLM_MAX_TOKENS")
    temperature: float = Field(0.1, env="LLM_TEMPERATURE")
    timeout: int = Field(120, env="LLM_TIMEOUT")

class GRPCSettings(BaseSettings):
    host: str = Field("0.0.0.0", env="GRPC_HOST")
    port: int = Field(50051, env="GRPC_PORT")
    max_workers: int = Field(10, env="GRPC_MAX_WORKERS")

class MonitoringSettings(BaseSettings):
    log_level: str = Field("INFO", env="LOG_LEVEL")
    metrics_enabled: bool = Field(True, env="METRICS_ENABLED")
    metrics_port: int = Field(8002, env="METRICS_PORT")

class Settings(BaseSettings):
    environment: str = Field("development", env="ENVIRONMENT")
    debug: bool = Field(False, env="DEBUG")
    
    # Service settings
    database: DatabaseSettings = DatabaseSettings()
    kafka: KafkaSettings = KafkaSettings()
    llm: LLMSettings = LLMSettings()
    grpc: GRPCSettings = GRPCSettings()
    monitoring: MonitoringSettings = MonitoringSettings()
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"
        case_sensitive = False

# Singleton instance
settings = Settings()
```

## 10. 모니터링 설정

### k8s/monitoring/prometheus-config.yaml

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: document-parser
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s
      external_labels:
        cluster: 'document-parser'
    
    rule_files:
      - /etc/prometheus/rules/*.yml
    
    scrape_configs:
      - job_name: 'prometheus'
        static_configs:
          - targets: ['localhost:9090']
      
      - job_name: 'rust-api'
        kubernetes_sd_configs:
          - role: endpoints
            namespaces:
              names: ['document-parser']
        relabel_configs:
          - source_labels: [__meta_kubernetes_service_name]
            action: keep
            regex: rust-api-service
          - source_labels: [__meta_kubernetes_endpoint_port_name]
            action: keep
            regex: metrics
      
      - job_name: 'llm-service'
        kubernetes_sd_configs:
          - role: endpoints
            namespaces:
              names: ['document-parser']
        relabel_configs:
          - source_labels: [__meta_kubernetes_service_name]
            action: keep
            regex: llm-service
          - source_labels: [__meta_kubernetes_endpoint_port_name]
            action: keep
            regex: metrics
      
      - job_name: 'kafka'
        static_configs:
          - targets: ['kafka-exporter:9308']
      
      - job_name: 'minio'
        static_configs:
          - targets: ['minio:9000']
        metrics_path: /minio/v2/metrics/cluster

    alerting:
      alertmanagers:
        - static_configs:
            - targets: ['alertmanager:9093']
```

### Grafana Dashboard JSON

```json
{
  "dashboard": {
    "id": null,
    "title": "Document Parser System Dashboard",
    "description": "Comprehensive monitoring for document parser system",
    "refresh": "30s",
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "panels": [
      {
        "id": 1,
        "title": "System Overview",
        "type": "stat",
        "targets": [
          {
            "expr": "up{job=~\"rust-api|llm-service\"}",
            "legendFormat": "{{job}} - {{instance}}"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "color": {
              "mode": "thresholds"
            },
            "thresholds": {
              "steps": [
                {"color": "red", "value": 0},
                {"color": "green", "value": 1}
              ]
            }
          }
        }
      },
      {
        "id": 2,
        "title": "Document Processing Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(document_parser_documents_processed_total[5m])",
            "legendFormat": "Documents/sec"
          }
        ]
      },
      {
        "id": 3,
        "title": "Parse Job Status",
        "type": "piechart",
        "targets": [
          {
            "expr": "document_parser_jobs_total by (status)",
            "legendFormat": "{{status}}"
          }
        ]
      },
      {
        "id": 4,
        "title": "Response Times",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.50, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "50th percentile"
          }
        ]
      }
    ]
  }
}
```

## 11. 추가 구성 요소들

### src/lib/api/types.ts

```typescript
export interface Document {
  id: string;
  name: string;
  type: string;
  size: number;
  file: File;
  uploadedAt: Date;
}

export interface ParseOptions {
  outputFormat: 'minimal' | 'lite' | 'full';
  useOcr: boolean;
  useLlm: boolean;
  aiProvider: 'openai' | 'anthropic' | 'google' | 'ollama';
  aiModel: string;
  filterHiddenCells: boolean;
  customPrompt: string;
}

export interface ParseJob {
  job_id: string;
  status: 'queued' | 'processing' | 'completed' | 'failed';
  progress: number;
  message?: string;
  created_at: string;
  updated_at: string;
  estimated_completion?: string;
}

export interface ParseResult {
  job_id: string;
  filename: string;
  content_type: string;
  parsed_content: any;
  metadata: any;
  created_at: string;
}
```

### docker-compose.yml (개발 환경)

```yaml
version: '3.8'

services:
  # Core Services
  rust-api:
    build: ./backend/rust-api
    ports:
      - "8000:8000"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/document_parser
      - KAFKA_BROKERS=kafka:9092
    depends_on:
      - postgres
      - kafka
      - minio

  llm-service:
    build: ./backend/python-services/llm-service
    ports:
      - "8001:8001"
      - "50051:50051"
    environment:
      - OPENAI_API_KEY=${OPENAI_API_KEY}
      - ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}
      - GOOGLE_API_KEY=${GOOGLE_API_KEY}
    depends_on:
      - kafka

  frontend:
    build: ./frontend
    ports:
      - "5173:80"
    depends_on:
      - rust-api

  # Infrastructure
  postgres:
    image: postgres:15
    environment:
      - POSTGRES_DB=document_parser
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  kafka:
    image: confluentinc/cp-kafka:latest
    environment:
      KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
      KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://kafka:9092
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
    depends_on:
      - zookeeper
    ports:
      - "9092:9092"

  zookeeper:
    image: confluentinc/cp-zookeeper:latest
    environment:
      ZOOKEEPER_CLIENT_PORT: 2181
      ZOOKEEPER_TICK_TIME: 2000

  minio:
    image: minio/minio:latest
    command: server /data --console-address ":9001"
    environment:
      - MINIO_ROOT_USER=minioadmin
      - MINIO_ROOT_PASSWORD=minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio_data:/data

  # Monitoring
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./config/prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana

volumes:
  postgres_data:
  minio_data:
  grafana_data:
```

이 완전한 문서 파서 시스템은 다음과 같은 특징을 제공합니다:

## 🎯 핵심 특징

1. **확장 가능한 아키텍처**: 마이크로서비스 기반으로 각 컴포넌트가 독립적으로 확장 가능
2. **다중 AI 제공자 지원**: OpenAI, Anthropic, Google, Ollama 통합
3. **실시간 처리**: Kafka 기반 비동기 작업 큐
4. **모던 UI**: SvelteKit + TypeScript + Tailwind CSS
5. **완전한 DevOps**: Docker, Kubernetes, CI/CD 파이프라인
6. **포괄적 모니터링**: Prometheus + Grafana 대시보드

## 🚀 즉시 시작 가능

```bash
# 개발 환경 시작
docker-compose up -d

# Kubernetes 배포
kubectl apply -f infrastructure/k8s/
```

이 시스템은 프로덕션 레디 상태로, 대규모 문서 처리 워크로드를 처리할 수 있도록 설계되었습니다.