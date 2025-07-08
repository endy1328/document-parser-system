FROM rust:1.82 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM rust:1.82
WORKDIR /app
COPY --from=builder /app/target/release/rust-api /app/rust-api
EXPOSE 8080
CMD ["/app/rust-api"]
