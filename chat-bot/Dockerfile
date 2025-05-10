FROM rust:1.86 AS builder
WORKDIR /app

COPY Cargo.toml ./
RUN mkdir src     && echo 'fn main() { println!("hello"); }' > src/main.rs     && cargo fetch     && rm -rf src

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/chat-bot /usr/local/bin/chat-bot
ENTRYPOINT ["/usr/local/bin/chat-bot"]
