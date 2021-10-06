FROM rust:1-slim-bullseye as builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update \
    && apt install --yes --no-install-recommends musl-tools \
    && apt-get clean

WORKDIR /app

COPY deps/destream_json/Cargo.toml deps/destream_json/
COPY Cargo.* ./
RUN mkdir -p "deps/destream_json/src" \
    && echo "fn main() {}" > deps/destream_json/src/main.rs \
    && mkdir -p "src" \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --target x86_64-unknown-linux-musl --release

COPY . ./
RUN cargo build --target x86_64-unknown-linux-musl --release && strip target/x86_64-unknown-linux-musl/release/big-brother

FROM debian:bullseye-slim

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/big-brother /big-brother

CMD [ "/big-brother" ]
