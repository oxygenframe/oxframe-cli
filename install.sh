#!/bin/sh
# Installs the `oxframe` CLI from GitHub releases.
#   curl -fsSL https://github.com/oxygenframe/oxframe-cli/releases/latest/download/install.sh | sh
# Env: OXFRAME_VERSION (default: latest), OXFRAME_INSTALL_DIR (default: ~/.local/bin).
set -eu
repo=oxygenframe/oxframe-cli
dir=${OXFRAME_INSTALL_DIR:-$HOME/.local/bin}
os=$(uname -s); arch=$(uname -m)
case "$os-$arch" in
  Linux-x86_64) target=x86_64-unknown-linux-gnu ;;
  Linux-aarch64|Linux-arm64) target=aarch64-unknown-linux-gnu ;;
  Darwin-x86_64) target=x86_64-apple-darwin ;;
  Darwin-arm64) target=aarch64-apple-darwin ;;
  *) echo "unsupported platform $os-$arch; use: cargo install oxframe-cli" >&2; exit 1 ;;
esac
if [ -n "${OXFRAME_VERSION:-}" ]; then tag=$OXFRAME_VERSION; else
  tag=$(curl -fsSL "https://api.github.com/repos/$repo/releases/latest" | sed -n 's/.*"tag_name": "\(.*\)".*/\1/p')
fi
name="oxframe-$tag-$target"
base="https://github.com/$repo/releases/download/$tag"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
echo "downloading oxframe $tag for $target"
curl -fsSL "$base/$name.tar.gz" -o "$tmp/m.tgz"
curl -fsSL "$base/$name.tar.gz.sha256" -o "$tmp/m.sha256"
expected=$(cut -d' ' -f1 "$tmp/m.sha256")
actual=$( (sha256sum "$tmp/m.tgz" 2>/dev/null || shasum -a 256 "$tmp/m.tgz") | cut -d' ' -f1)
[ "$expected" = "$actual" ] || { echo "checksum mismatch" >&2; exit 1; }
tar -xzf "$tmp/m.tgz" -C "$tmp"
mkdir -p "$dir" && install -m 755 "$tmp/$name/oxframe" "$dir/oxframe"
echo "installed $dir/oxframe"
case ":$PATH:" in *":$dir:"*) ;; *) echo "add $dir to your PATH" ;; esac
