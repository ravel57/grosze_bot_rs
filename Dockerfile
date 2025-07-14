FROM rust:latest

COPY ./ ./

RUN cargo build --release

CMD ["./target/release/grosze_bot_rs"]