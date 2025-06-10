FROM rust:slim AS chef

# install build tools
RUN apt-get update -y && \
    apt-get install -y pkg-config make g++ libssl-dev musl-tools && \
    rustup target add x86_64-unknown-linux-musl

# add cargo chef (this installs only once on first build)
RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# build docker caching layer
RUN cargo chef cook --release --recipe-path recipe.json

# build binary
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# final scratch image; launch app
FROM scratch
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/vectorize-project /vectorize-project
ENTRYPOINT [ "/vectorize-project" ]
