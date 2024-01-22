FROM rust as builder

ENV SQLX_OFFLINE=true

WORKDIR /aodata-api
COPY Cargo.toml .
COPY Cargo.lock .
RUN \
    mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rvf src
    
COPY . .
RUN cargo build --release

FROM ubuntu:latest

ENV ENV=PROD
EXPOSE 8080

COPY --from=builder /aodata-api/target/release/aodata-api /usr/local/bin/aodata-api

CMD ["aodata-api"]