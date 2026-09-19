# Bivdi Runtime — container image (D-013)
#
# The runtime is container-friendly from the first executable, but containers
# are a deployment/compatibility convenience, never a Bivdi primitive.

# ---- build stage ----------------------------------------------------------
# MSRV is 1.88 (workspace `rust-version` + `runtime/rust-toolchain.toml`);
# use the matching image so `--locked` can resolve ICU/wasmtime transitive deps.
FROM rust:1.88 AS build
WORKDIR /src
COPY runtime/ .
RUN cargo build --workspace --locked --release

# ---- runtime stage ---------------------------------------------------------
FROM debian:bookworm-slim
RUN useradd --create-home --uid 1000 bivdi
USER bivdi
COPY --from=build /src/target/release/bivdi-cli /usr/local/bin/bivdi-cli
ENTRYPOINT ["bivdi-cli"]
