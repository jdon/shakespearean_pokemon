FROM rust as planner
WORKDIR /app
RUN cargo install cargo-chef 
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust as cacher
WORKDIR /app
RUN cargo install cargo-chef
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as builder
WORKDIR /app
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin shakespearean_pokemon

FROM debian:buster-slim as runtime
RUN apt-get update && apt-get -y install ca-certificates libssl-dev libcurl4-openssl-dev
WORKDIR /app
COPY --from=builder /app/target/release/shakespearean_pokemon /usr/local/bin
ENTRYPOINT ["/usr/local/bin/shakespearean_pokemon"]