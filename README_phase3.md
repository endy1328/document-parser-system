# Phase 3: 비동기 작업 큐 및 상태 추적

## 🎯 목표
- 대용량/장시간 문서 파싱을 위한 비동기 아키텍처 도입
- 작업 큐(Redis) 기반으로 업로드/파싱 분리
- 실시간 상태/진행률/결과 조회 API 제공

---

## ✅ 구현 내역

### Backend (Rust)
- 작업(Job) 데이터 모델 및 상태(JobStatus, JobResult) 정의
- Redis 연동 작업 큐 서비스(JobQueue) 구현 (enqueue, get_job, update_job_status)
- 업로드 핸들러 비동기 처리로 리팩토링 - 작업 큐에 등록 후 즉시 job_id 반환
- 백그라운드 워커 구현 - 주기적으로 큐 폴링 및 작업 처리 (별도 스레드에서 실행)
- 작업 상태/결과 조회 API 구현 (`/jobs/{job_id}`, `/jobs/{job_id}/result`)
- Redis URL 환경변수 지원, 로깅 기능 강화

### Frontend (Svelte)
- (예정) 업로드 후 job_id로 상태 폴링/구독, 진행률/결과 UI 표시

### Infra
- Docker Compose에 Redis 서비스 추가 필요

---

## 🚀 실행 및 테스트 방법

### 1. Redis 서비스 추가 (docker-compose.dev.yml)
```yaml
services:
  redis:
    image: redis:7
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  rust-api:
    # 기존 설정 유지
    environment:
      - REDIS_URL=redis://redis:6379/
    depends_on:
      - redis

volumes:
  redis_data:
```

### 2. 백엔드/프론트엔드/Redis 모두 실행
```bash
cd ~/projects/document-parser-system
docker-compose -f docker-compose.dev.yml up --build -d
```

### 3. API 테스트 (업로드/상태/결과)

#### 1) 파일 업로드 (비동기 큐 등록)
```bash
curl -F "file=@sample.pdf" http://localhost:8080/upload
```
- 응답 예시:
```json
{
  "status": "accepted",
  "job_id": "b2e1c3b2-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "filename": "sample.pdf",
  "file_type": "pdf",
  "message": "파일이 업로드되어 처리 대기열에 등록되었습니다."
}
```

#### 2) 작업 상태 조회
```bash
curl http://localhost:8080/jobs/{job_id}
```
- 응답 예시:
```json
{
  "job_id": "b2e1c3b2-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "status": "processing",
  "progress": 40,
  "message": "4/10 페이지 처리 중..."
}
```

#### 3) 작업 결과 조회
```bash
curl http://localhost:8080/jobs/{job_id}/result
```
- 응답 예시:
```json
{
  "job_id": "b2e1c3b2-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "result": { ...파싱 결과... }
}
```

---

## 🛠️ 참고/주의 사항
- 반드시 Redis가 실행 중이어야 정상 동작
- 현재는 skeleton 구조(큐 등록/상태 조회 API만 동작, 워커/실파싱 미구현)
- 실제 파싱/진행률/결과 저장은 다음 단계에서 구현 예정
- 프론트엔드 연동은 Phase 3 후반에 진행

---

## 🔄 다음 단계
- 프론트엔드 상태 폴링/진행률 표시 구현
- 에러/예외 처리 강화 및 모니터링 기능 추가
- 작업 히스토리 및 관리 기능 구현 (작업 취소, 재시도 등)
- 대용량 파일 처리 최적화 (스트리밍, 청크 단위 처리 등)

---

## 📁 관련 파일
- `src/models/job.rs`: 작업/상태/결과 모델
- `src/services/job_queue.rs`: Redis 큐 서비스
- `src/services/worker.rs`: 백그라운드 작업 처리 워커
- `src/handlers/job.rs`: 상태/결과 API 핸들러
- `src/handlers/upload.rs`: 업로드 API 핸들러 (비동기 처리)
- `main.rs`: 워커 스레드 실행, 서비스/라우터 등록

---

## 💬 문의/기여
- 이 단계의 구조/테스트/확장에 대한 피드백을 환영합니다!
