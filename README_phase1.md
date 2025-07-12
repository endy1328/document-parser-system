# Phase 1: 기본 파일 업로드 및 PDF 텍스트 추출

## 🎯 목표
단순한 파일 업로드 및 PDF 텍스트 추출 기능 구현

## ✅ 완료된 기능

### Backend (Rust)
- `/upload` POST 엔드포인트 구현
- PDF 텍스트 추출 (pdftotext 외부 명령어 사용)
- 기본 JSON 응답 구조
- 파일 저장 (로컬 파일시스템)
- CORS 설정 추가

### Frontend (Svelte + Vite)
- 파일 업로드 UI 구현
- 업로드 진행률 표시
- 결과 JSON 표시
- 에러 처리 및 사용자 피드백

### Docker 환경
- 멀티 컨테이너 구성 (rust-api, frontend)
- 개발 환경 Docker Compose 설정
- poppler-utils 설치 (PDF 처리용)

## 🚀 사용 방법

### 1. 시스템 실행
```bash
cd ~/projects/document-parser-system
docker-compose -f docker-compose.dev.yml up --build -d
```

### 2. 웹 UI 접속
- 브라우저에서 `http://localhost:3000` 접속
- PDF 파일을 드래그하거나 "파일 선택" 버튼으로 업로드
- 업로드 진행률 및 결과 확인

### 3. API 직접 호출
```bash
curl -F "file=@sample.pdf" http://localhost:8080/upload
```

## 📋 API 스펙

### POST /upload
**요청:**
- Content-Type: `multipart/form-data`
- 필드명: `file`

**응답 (성공):**
```json
{
  "status": "success",
  "filename": "sample.pdf",
  "text": "(PDF에서 추출된 텍스트)"
}
```

**응답 (실패):**
```json
{
  "status": "fail",
  "filename": "sample.pdf",
  "error": "pdftotext 오류: ..."
}
```

## 🔧 기술 스택

### Backend
- Rust 1.82
- actix-web 4
- actix-multipart 0.6
- actix-cors 0.7
- poppler-utils (pdftotext)

### Frontend
- Svelte 4
- Vite 5
- Node.js 20

### 인프라
- Docker
- Docker Compose

## 📁 프로젝트 구조
```
document-parser-system/
├── rust-api/
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── uploads/          # 업로드된 파일 저장
├── frontend/
│   ├── src/
│   │   ├── App.svelte
│   │   └── main.js
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── Dockerfile.dev
├── docker-compose.dev.yml
└── README_phase1.md
```

## 🎯 완료 기준
- [x] PDF 파일을 업로드하면 텍스트가 추출됨
- [x] 간단한 UI에서 결과 확인 가능
- [x] 에러 처리 기본 구현
- [x] Docker 환경에서 정상 실행
- [x] CORS 문제 해결

## 🐛 알려진 제한사항
- PDF 파일만 텍스트 추출 지원
- 다른 포맷은 저장만 가능
- 동기 처리로 인한 대용량 파일 처리 제한
- 기본적인 UI/UX

## 🔄 다음 단계 (Phase 2)
- 다중 포맷 지원 (DOCX, XLSX, TXT, MD)
- 백엔드 코드 모듈화
- 향상된 프론트엔드 UI
- 파일 타입별 파싱 결과 구조화
