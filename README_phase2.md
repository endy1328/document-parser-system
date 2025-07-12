# Phase 2: 다중 포맷 지원 및 UI 개선

## 🎯 목표
PDF 외 다양한 문서 포맷(DOCX, XLSX, TXT, MD) 지원 및 프론트엔드 UI 개선

## ✅ 완료된 기능

### Backend (Rust)
- PDF, DOCX, XLSX, TXT, MD 파일 파싱 지원
- 파일 타입별 파싱 로직 분리 (서비스/핸들러/모델 모듈화)
- 파싱 결과를 구조화된 JSON으로 반환
- 파일 타입 자동 감지 및 검증
- 업로드/파싱 에러 시 상세 메시지 반환
- docx2txt, xlsx2csv 등 외부 도구 연동 (Dockerfile에 설치)

### Frontend (Svelte + Vite)
- 파일 타입별 아이콘 및 색상 표시
- 지원 포맷 안내 UI
- 파싱 결과(텍스트/스프레드시트) 구조화 렌더링
- 파일 메타데이터(크기, 생성 시간 등) 표시
- 에러 메시지/상태 메시지 개선

### Docker 환경
- docx2txt, xlsx2csv 등 다중 포맷 지원 도구 설치
- 헬스체크 추가

## 🚀 사용 방법

### 1. 시스템 실행
```bash
cd ~/projects/document-parser-system
docker-compose -f docker-compose.dev.yml up --build -d
```

### 2. 웹 UI 접속
- `http://localhost:3000`에서 다양한 포맷 문서 업로드 및 결과 확인
- 파일별로 아이콘/색상/메타데이터/미리보기 제공

### 3. API 직접 호출
```bash
curl -F "file=@sample.xlsx" http://localhost:8080/upload
curl -F "file=@sample.docx" http://localhost:8080/upload
curl -F "file=@sample.txt" http://localhost:8080/upload
curl -F "file=@sample.md" http://localhost:8080/upload
```

## 📋 개선된 API 스펙

### POST /upload
**요청:**
- Content-Type: `multipart/form-data`
- 필드명: `file`

**응답 (성공):**
```json
{
  "status": "success",
  "id": "uuid",
  "filename": "sample.xlsx",
  "file_type": "xlsx",
  "content": {
    "sheets": [
      {
        "name": "Sheet1",
        "rows": [["A1","B1"],["A2","B2"]]
      }
    ],
    "metadata": {
      "file_type": "xlsx",
      "file_size": 12345,
      "created_at": "2025-07-10T15:15:00+09:00"
    }
  },
  "message": "XLSX 파일이 성공적으로 파싱되었습니다."
}
```

**응답 (에러):**
```json
{
  "status": "error",
  "filename": "sample.docx",
  "file_type": "docx",
  "error": "지원하지 않는 파일 형식입니다.",
  "message": "파일 파싱 중 오류가 발생했습니다."
}
```

## 🔧 기술 스택 및 주요 변경점

### Backend
- Rust 1.82
- actix-web 4, actix-cors, actix-multipart
- serde, uuid, chrono
- 외부 도구: poppler-utils, docx2txt, python3, xlsx2csv

### Frontend
- Svelte 4, Vite 5
- 파일별 미리보기/메타데이터/구조화 UI

### 인프라
- Docker, Docker Compose
- 멀티 컨테이너 (rust-api, frontend)

## 📁 프로젝트 구조 (Phase 2)
```
document-parser-system/
├── rust-api/
│   ├── src/
│   │   ├── handlers/
│   │   │   ├── health.rs
│   │   │   ├── upload.rs
│   │   │   └── mod.rs
│   │   ├── models/
│   │   │   ├── document.rs
│   │   │   └── mod.rs
│   │   ├── services/
│   │   │   ├── parser.rs
│   │   │   └── mod.rs
│   │   └── main.rs
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── uploads/
├── frontend/
│   ├── src/
│   │   ├── App.svelte
│   │   └── main.js
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── Dockerfile.dev
├── docker-compose.dev.yml
├── README_phase2.md
└── README.md
```

## 🎯 완료 기준
- [x] PDF, DOCX, XLSX, TXT, MD 모두 파싱 가능
- [x] 각 포맷별로 구조화된 데이터 반환
- [x] UI에서 포맷별 미리보기 제공
- [x] 에러/메타데이터/상태 메시지 개선
- [x] Docker 환경에서 정상 실행

## 🐛 알려진 제한사항
- 일부 포맷은 외부 도구에 의존 (docx2txt, xlsx2csv)
- 대용량 파일/비동기 처리 미지원 (Phase 3 예정)
- 고급 미리보기/편집 기능 미구현

## 🔄 다음 단계 (Phase 3)
- 대용량/비동기 처리 (작업 큐, 상태 추적)
- 실시간 진행률 표시
- 작업 히스토리/다운로드/썸네일 등 고급 기능
