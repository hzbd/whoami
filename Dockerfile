# Stage 1: Builder
FROM rust:1-bullseye AS builder

RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/app
COPY . .

WORKDIR /home/rust/src
COPY . .
RUN cargo build --release --locked --target x86_64-unknown-linux-musl


# Stage 2: Final Image
FROM alpine:3.22
LABEL maintainer="623 <hello@sconts.com>"

ENV PORT 8000
EXPOSE 8000
COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/whoami /usr/local/bin/whoami

CMD ["whoami"]
