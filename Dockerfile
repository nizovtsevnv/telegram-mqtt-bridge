FROM rust:alpine
WORKDIR /mnt/src
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static
