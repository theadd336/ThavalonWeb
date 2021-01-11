FROM rust:1.46 as build

WORKDIR /app

COPY ./thavalon-server .

RUN cargo build --release

RUN mkdir -p /build-out/
RUN cp target/release/thavalon-server /build-out/

FROM ubuntu:latest
ENV DEBIAN_FRONTEND=noninteractive
# Below is required since these are needed at runtime.
RUN apt-get update && apt-get -y install ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=build /build-out/thavalon-server /

CMD [ "/thavalon-server" ]
