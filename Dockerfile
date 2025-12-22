FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

# Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

# Runtime
FROM debian:trixie-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get upgrade -y \
  # Clean up
  && apt-get autoremove -y \
  && apt-get clean -y \
  # Install curl for self health check
  && apt-get -y install curl \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder  /app/target/release/maedic maedic
COPY base.yaml base.yaml
EXPOSE 3000

ENTRYPOINT [ "./maedic" ]
