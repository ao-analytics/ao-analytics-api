FROM lukemathwalker/cargo-chef:latest as planner

WORKDIR /ao-analytics-api
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM  lukemathwalker/cargo-chef:latest as builder

ENV SQLX_OFFLINE=true
WORKDIR /ao-analytics-api
COPY --from=planner /ao-analytics-api/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM ubuntu:latest

COPY --from=builder /ao-analytics-api/target/release/ao-analytics-api /usr/local/bin/ao-analytics-api

CMD ["ao-analytics-api"]