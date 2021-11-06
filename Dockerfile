FROM rust:1.55.0 as builder

FROM builder as server-build
WORKDIR /usr/src/badbee
# advice from https://stackoverflow.com/questions/58473606/cache-rust-dependencies-with-docker-build
COPY dummy.rs .
COPY ./Cargo.* .
COPY ./backend/Cargo.toml ./backend/Cargo.toml
COPY ./web-server/Cargo.toml ./web-server/Cargo.toml

RUN sed -i 's#src/lib.rs#../dummy.rs#' backend/Cargo.toml
RUN cargo build --release -p badbee-backend

RUN sed -i 's#src/main.rs#../dummy.rs#' web-server/Cargo.toml
RUN cargo build --release -p web-server

WORKDIR /usr/src/badbee
COPY ./web-server ./web-server
COPY ./backend ./backend

RUN cargo build --release

FROM builder as client-build
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
WORKDIR /usr/src/badbee
COPY ./web-client ./web-client
COPY ./static ./static
WORKDIR /usr/src/badbee/web-client
RUN wasm-pack build --out-dir ../static --target web --release

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/badbee
COPY --from=server-build /usr/src/badbee/target/release/web-server .
COPY --from=client-build /usr/src/badbee/static ./static
COPY font1.png .
CMD ["./web-server"]