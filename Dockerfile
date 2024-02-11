FROM rust as builder

ENV SQLX_OFFLINE=true

WORKDIR /ao-analytics-api
COPY . .
COPY .env.prod .env
RUN cargo install --path .

FROM ubuntu:latest

EXPOSE 8080

COPY --from=builder /usr/local/cargo/bin/ao-analytics-api /usr/local/bin/ao-analytics-api

CMD ["ao-analytics-api"]