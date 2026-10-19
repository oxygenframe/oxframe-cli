#!/usr/bin/env bash
# Downloads the editor-ui release pinned in ui.lock into ui/dist (no network
# access happens in build.rs or `cargo install`). `latest` ignores the pin
# (nightly cross-repo runs).
set -euo pipefail
cd "$(dirname "$0")/.."
if [ "${1:-}" = latest ]; then
  url=$(curl -fsSL https://api.github.com/repos/oxygenframe/oxframe-web/releases/latest \
    | sed -n 's/.*"browser_download_url": "\(.*editor-ui-.*\.tar\.gz\)".*/\1/p' | head -1)
  sum=""
else
  url=$(sed -n 's/^url = "\(.*\)"/\1/p' ui.lock)
  sum=$(sed -n 's/^sha256 = "\(.*\)"/\1/p' ui.lock)
fi
tmp=$(mktemp -d)
curl -fsSL "$url" -o "$tmp/ui.tgz"
if [ -n "$sum" ]; then echo "$sum  $tmp/ui.tgz" | sha256sum -c -; fi
rm -rf ui/dist && mkdir -p ui/dist && tar -xzf "$tmp/ui.tgz" -C ui/dist
echo "editor-ui from $url → ui/dist"
