FROM rust:latest AS builder

WORKDIR /rust/src/app
COPY . .

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl
RUN strip ./target/x86_64-unknown-linux-musl/release/simulation_server-v2

FROM alpine
WORKDIR /app

COPY --from=builder /rust/src/app/target/x86_64-unknown-linux-musl/release/simulation_server-v2 /app/

CMD ["./simulation_server-v2"]