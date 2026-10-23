# oxframe in a container: the oxframe CLI, Rust and ffmpeg, for CI and for
# cloud agents. The SDK is compiled once into a shared target directory, so
# a project's first build in the container compiles only its own scenes.
#
#   docker run --rm -v "$PWD:/work" ghcr.io/oxygenframe/oxframe oxframe render
ARG RUST=1.97.1
FROM docker.io/library/rust:${RUST}-slim-bookworm

ARG OXFRAME_VERSION
RUN test -n "$OXFRAME_VERSION" \
 && apt-get update \
 && apt-get install -y --no-install-recommends ffmpeg ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# The CLI from crates.io, built against this image's libc.
RUN cargo install oxframe-cli --version "$OXFRAME_VERSION" --locked --root /usr/local \
 && rm -rf "$CARGO_HOME/registry/src" \
 && oxframe --version

# Every project shares this target directory: the engine it compiles here is
# reused by the first build of yours.
# No debug info in check builds: a third of the size, and nobody runs a
# debugger in CI. As an ENV it holds for your builds too, so they match the
# cache.
ENV CARGO_TARGET_DIR=/var/cache/oxframe/target \
    CARGO_PROFILE_DEV_DEBUG=0
RUN cd /tmp \
 && oxframe init warm \
 && cd warm \
 && oxframe check \
 && oxframe render --format png --range 0..0.1 \
 && cd / && rm -rf /tmp/warm "$CARGO_TARGET_DIR"/*/warm* "$CARGO_TARGET_DIR"/*/deps/warm-* \
 && chmod -R a+rwX /var/cache/oxframe "$CARGO_HOME"

WORKDIR /work
