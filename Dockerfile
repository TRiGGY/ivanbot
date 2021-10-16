FROM rust:1.51 as cargo-build
ENV IVAN_CONNECT_IP "127.0.0.1:9000"
ENV IVAN_PASSWORD "your_rcon_password"
ENV ADMIN_ID "Discord_id_for_admin"
ENV CONFIG_PATH "/root/ivan.json"
ENV ALLOW_USERS = "false"

WORKDIR /usr/src/ivanbot

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN cargo build --release

RUN rm -f target/release/deps/ivanbot*

COPY . .

RUN cargo build --release

RUN cargo install --path .

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM ubuntu:latest

COPY --from=cargo-build /usr/local/cargo/bin/ivanbot /usr/local/bin/ivanbot

CMD ["/usr/local/bin/ivanbot"]