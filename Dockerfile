FROM docker.io/rust:1.63 as builder
WORKDIR /usr/src/beverley
COPY . .
RUN cargo install --path .

FROM docker.io/debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/beverley /usr/local/bin/beverley
CMD ["beverley"]