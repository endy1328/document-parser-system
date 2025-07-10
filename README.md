# document-parser-system

## Docker 실행
```bash
cd ~/projects/document-parser-system
docker-compose -f docker-compose.dev.yml down
docker-compose -f docker-compose.dev.yml up --build -d
```

## Phase 1: 기본 파일 업로드 및 PDF 텍스트 추출

### 주요 기능
- `/upload` 엔드포인트에서 파일 업로드(multipart/form-data) 지원
- 업로드된 파일을 `rust-api/uploads/` 폴더에 저장
- PDF 파일은 **외부 명령어 `pdftotext`(poppler-utils)**로 텍스트를 추출하여 JSON으로 반환
- PDF가 아닌 파일은 저장만 하고, 별도 메시지 반환
- **프론트엔드(SvelteKit)에서 파일 업로드/진행률/결과 표시 UI 구현**

### 사용 방법
#### 1. 웹 UI로 업로드 및 결과 확인
- 브라우저에서 `http://localhost:3000` 접속
- PDF 파일을 드래그하거나 "파일 선택" 버튼으로 업로드
- 업로드 진행률 및 결과(JSON/텍스트) 화면에서 바로 확인 가능

#### 2. API 직접 호출 (curl 예시)
- URL: `POST /upload`
- Content-Type: `multipart/form-data`
- 필드명: `file`

```bash
curl -F "file=@sample.pdf" http://localhost:8080/upload
curl -F "file=@business_proposal.pdf" http://localhost:8080/upload
```

### 에러 처리
- PDF 파싱 실패, pdftotext 실행 오류, 네트워크 오류 등은 모두 JSON 형태로 명확하게 안내됩니다.
- 예시:
```json
{
  "status": "fail",
  "filename": "document.pdf",
  "error": "pdftotext 오류: ..."
}
```

#### 응답 예시
- PDF 파일 성공 시:
```json
{
  "status": "success",
  "filename": "sample.pdf",
  "text": "(PDF에서 추출된 텍스트)"
}
```
- PDF 파싱 실패 시:
```json
{
  "status": "fail",
  "filename": "sample.pdf",
  "error": "PDF 파싱 실패: ..."
}
```
- PDF가 아닌 파일:
```json
{
  "status": "success",
  "filename": "sample.txt",
  "message": "PDF가 아닌 파일은 저장만 합니다."
}
```

### 의존성
- actix-web
- actix-multipart
- tokio
- futures-util
- serde_json
- **poppler-utils (시스템 패키지, Dockerfile에서 설치 필요)**

### 기타
- 업로드 폴더: `rust-api/uploads/` (없으면 자동 생성)
- 헬스체크: `GET /healthz` (OK 반환)

---

#### [중요]
- PDF 텍스트 추출은 Rust 코드에서 **외부 명령어 `pdftotext`(poppler-utils)**를 호출하여 처리합니다.
- Dockerfile에 아래와 같이 패키지 설치가 필요합니다:

```dockerfile
RUN apt-get update && apt-get install -y poppler-utils
```

---

**각 단계별 구현 내역 및 사용법은 README.md에 계속 기록됩니다.**
