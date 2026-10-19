# Embedding the editor: ui/dist is what `rust-embed` packs into the binary.

# Download the pinned editor-ui release (see ui.lock) into ui/dist.
fetch-ui:
    #!/usr/bin/env bash
    set -euo pipefail
    url=$(sed -n 's/^url = "\(.*\)"/\1/p' ui.lock)
    sum=$(sed -n 's/^sha256 = "\(.*\)"/\1/p' ui.lock)
    tmp=$(mktemp -d)
    curl -fsSL "$url" -o "$tmp/ui.tgz"
    if [ -n "$sum" ]; then echo "$sum  $tmp/ui.tgz" | sha256sum -c -; fi
    rm -rf ui/dist && mkdir -p ui/dist
    tar -xzf "$tmp/ui.tgz" -C ui/dist
    echo "editor-ui $(sed -n 's/^version = "\(.*\)"/\1/p' ui.lock) → ui/dist"

# Build the editor from a sibling oxframe-web checkout (cross-repo development).
build-ui web="../oxframe-web":
    cd {{web}} && bun install && bun run build:editor
    rm -rf ui/dist && mkdir -p ui/dist
    cp -r {{web}}/packages/editor-ui/dist/. ui/dist/
    @du -sh ui/dist

# Package ui/dist as the release tarball format (used by oxframe-web CI).
pack-ui out="editor-ui.tar.gz":
    tar -czf {{out}} -C ui/dist .
    sha256sum {{out}}
