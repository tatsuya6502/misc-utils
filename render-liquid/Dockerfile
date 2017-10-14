FROM ekidd/rust-musl-builder AS builder

ADD . ./
RUN cargo test
RUN cargo build --release
RUN strip target/x86_64-unknown-linux-musl/release/render-liquid

FROM scratch
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/render-liquid \
    /usr/local/bin/

ENTRYPOINT ["/usr/local/bin/render-liquid"]
