FROM rust:1.43 as build

COPY src /opt/properwatcher/src
COPY Cargo.toml /opt/properwatcher
COPY Cargo.lock /opt/properwatcher
WORKDIR /opt/properwatcher
RUN cargo build --release

FROM debian

COPY --from=build /opt/properwatcher/target/release/properwatcher /opt/properwatcher/properwatcher

COPY config.sample.toml /opt/properwatcher/
RUN chmod +x /opt/properwatcher/properwatcher
WORKDIR /opt/properwatcher

RUN apt-get update && apt-get install -y libssl-dev ca-certificates

ENTRYPOINT [ "./properwatcher" ]