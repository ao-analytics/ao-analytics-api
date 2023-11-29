FROM rust as builder

ENV SQLX_OFFLINE=true

WORKDIR /aodata-api
COPY . .
RUN cargo install --path .

FROM ubuntu:latest

ENV ENV=PROD
EXPOSE 8080

COPY --from=builder /usr/local/cargo/bin/aodata-api /usr/local/bin/aodata-api

CMD ["aodata-api"]