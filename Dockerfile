FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR /app
RUN apt-get update \
  && apt-get install -y --no-install-recommends lld clang\
  && apt-get clean -y

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
RUN apt-get update -y \
  # && apt-get upgrade -y \
  # Clean up
  && apt-get autoremove -y \
  # Install curl for self health check
  && apt-get install -y --no-install-recommends curl \
  # Install libkrb5-dev for integrated auth via libgssapi
  && apt-get install -y --no-install-recommends libkrb5-dev \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

# Create a custom user with UID 1234 and GID 1234
RUN groupadd -g 1234 maedicgroup && \
    useradd -m -u 1234 -g maedicgroup maedic
USER maedic:maedicgroup
WORKDIR /app
COPY --from=builder --chown=maedic:maedicgroup  /app/target/release/maedic maedic
EXPOSE 3000
HEALTHCHECK --interval=60s --timeout=10s --retries=3 \
  CMD curl -f http://localhost:3000/v1/health || exit 1

ENTRYPOINT [ "./maedic" ]
