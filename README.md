# Document Parser System

문서 파서 시스템은 다양한 포맷(PDF, DOCX, XLSX, TXT, MD)의 문서를 업로드하면 서버에서 텍스트/테이블 데이터를 추출해 구조화된 결과로 반환하는 오픈소스 프로젝트입니다.

---

## 🚀 주요 특징 (최신)
- **다중 포맷 지원:** PDF, DOCX, XLSX, TXT, MD 파일 파싱
- **구조화된 결과:** 텍스트/스프레드시트/메타데이터 통합 응답
- **모듈화된 Rust 백엔드:** 핸들러, 서비스, 모델 분리
- **모던 Svelte 프론트엔드:** 파일별 아이콘/미리보기/메타데이터/상태 UX
- **Docker 기반 멀티컨테이너:** rust-api, frontend, 외부 도구 자동 설치
- **에러/상태/지원 포맷 안내 강화**

---

## 🗂️ 프로젝트 구조
```
document-parser-system/
├── rust-api/
│   ├── src/
│   │   ├── handlers/
│   │   ├── models/
│   │   └── services/
│   ├── Cargo.toml
│   ├── Dockerfile
│   └── uploads/
├── frontend/
│   ├── src/
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── Dockerfile.dev
├── docker-compose.dev.yml
├── README.md (최신 종합)
├── README_phase1.md
└── README_phase2.md
```

---

## ⚡ 빠른 시작
```bash
cd ~/projects/document-parser-system
docker-compose -f docker-compose.dev.yml up --build -d
```
- **프론트엔드**: http://localhost:3000
- **백엔드 API**: http://localhost:8080

---

## 🌐 웹 사용법
1. 웹에서 다양한 문서 파일 업로드 (PDF/DOCX/XLSX/TXT/MD)
2. 파일별 아이콘/색상/메타데이터/미리보기 자동 표시
3. 스프레드시트는 표로, 텍스트/마크다운은 원문으로 렌더링
4. 에러/상태/지원 포맷 안내 강화

---

## 📋 API 사용법
### POST /upload
- Content-Type: `multipart/form-data`
- 필드명: `file`

#### 예시 (curl)
```bash
curl -F "file=@sample.pdf" http://localhost:8080/upload
curl -F "file=@sample.xlsx" http://localhost:8080/upload
```

#### 응답 예시 (성공)
```json
{
  "status": "success",
  "id": "uuid",
  "filename": "sample.xlsx",
  "file_type": "xlsx",
  "content": {
    "sheets": [
      { "name": "Sheet1", "rows": [["A1","B1"],["A2","B2"]] }
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

#### 응답 예시 (에러)
```json
{
  "status": "error",
  "filename": "sample.docx",
  "file_type": "docx",
  "error": "지원하지 않는 파일 형식입니다.",
  "message": "파일 파싱 중 오류가 발생했습니다."
}
```

---

## 🛠️ 주요 기술 스택
- **Backend:** Rust 1.82, actix-web, actix-cors, actix-multipart, serde, uuid, chrono
- **외부 도구:** poppler-utils, docx2txt, python3, xlsx2csv
- **Frontend:** Svelte 4, Vite 5, Node.js 20
- **Infra:** Docker, Docker Compose

---

## 📝 단계별 상세 문서
- [README_phase1.md](./README_phase1.md): PDF 업로드/파싱 MVP
- [README_phase2.md](./README_phase2.md): 다중 포맷/구조화/프론트 고도화

---

## 🐛 제한 및 참고
- 일부 포맷은 외부 도구에 의존 (docx2txt, xlsx2csv)
- 대용량/비동기/고급 미리보기는 미지원 (Phase 3~)
- 상세 개발 로드맵 및 향후 계획은 [roadmap.md](./roadmap.md) 참고

---

## 💬 문의/기여
- 이 저장소는 오픈소스이며, 개선/기여/이슈 제보를 환영합니다!

---

**각 단계별 상세 내역은 README_phaseN.md에서 확인할 수 있습니다. 최신 사용법은 이 파일(README.md)을 참고하세요.**
