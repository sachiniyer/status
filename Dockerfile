FROM rust:1.70 as build

RUN USER=root cargo new --bin status
WORKDIR /status

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/status*
RUN cargo build --release

FROM rust:1.70

COPY --from=build /status/target/release/status .

CMD ["./status"]
