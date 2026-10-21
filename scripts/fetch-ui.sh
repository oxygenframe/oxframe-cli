#!/usr/bin/env bash
# Downloads the editor-ui release pinned in ui.lock into ui/dist (no network
# access happens in build.rs or `cargo install`). `latest` ignores the pin
# (nightly cross-repo runs).
set -euo pipefail
cd "$(dirname "$0")/.."
tmp=$(mktemp -d)
if [ "${1:-}" = latest ]; then
  # oxframe-web is private, so anonymous API calls get 404. gh reads GH_TOKEN,
  # which needs read access to oxframe-web's contents.
  gh release download --repo oxygenframe/oxframe-web --pattern 'editor-ui-*.tar.gz' \
    --output "$tmp/ui.tgz"
  url="oxygenframe/oxframe-web latest release"
else
  url=$(sed -n 's/^url = "\(.*\)"/\1/p' ui.lock)
  sum=$(sed -n 's/^sha256 = "\(.*\)"/\1/p' ui.lock)
  curl -fsSL "$url" -o "$tmp/ui.tgz"
  echo "$sum  $tmp/ui.tgz" | sha256sum -c -
fi
rm -rf ui/dist && mkdir -p ui/dist && tar -xzf "$tmp/ui.tgz" -C ui/dist
echo "editor-ui from $url → ui/dist"
