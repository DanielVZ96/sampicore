# Dockerfile for creating a statically-linked Rust application using docker's
# multi-stage build feature. This also leverages the docker build cache to avoid
# re-downloading dependencies if they have not changed.
FROM rustlang/rust:nightly-buster AS build
WORKDIR /usr/src

RUN rustup target add x86_64-unknown-linux-gnu
RUN apt update && apt install -y libxcb-randr0-dev

# Create a dummy project and build the app's dependencies.
# If the Cargo.toml or Cargo.lock files have not changed,
# we can use the docker build cache and skip these (typically slow) steps.
RUN USER=root cargo new sampic
WORKDIR /usr/src/sampic
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Copy the source and build the application.
RUN cargo install --target x86_64-unknown-linux-gnu --path .

# Copy the statically-linked binary into a scratch container.
FROM rustlang/rust:nightly-buster
WORKDIR /usr/src/sampic
RUN apt update && apt install -y libxcb-randr0-dev
RUN mkdir /.config && chown -R 1000 /.config
USER 1000
COPY --from=build /usr/local/cargo/bin/sampic /usr/src/sampic/
COPY ./entrypoint.sh /usr/src/sampic/entrypoint.sh
ENV PATH="/usr/src/sampic:${PATH}"
ENV XDG_CONFIG_HOME="/.config"
CMD ["./entrypoint.sh"]
