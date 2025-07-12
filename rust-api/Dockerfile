FROM rust:1.82 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# 다중 포맷 지원을 위한 도구들 설치
RUN apt-get update && apt-get install -y \
    poppler-utils \
    docx2txt \
    python3 \
    python3-pip \
    && pip3 install --break-system-packages xlsx2csv \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-api /app/rust-api
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/healthz || exit 1
CMD ["/app/rust-api"]
