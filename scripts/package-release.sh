#!/usr/bin/env bash
set -euo pipefail

version="${1:-$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')}"
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist="$repo_root/dist"
mkdir -p "$dist"

targets=(
  "x86_64-apple-darwin"
  "aarch64-apple-darwin"
  "x86_64-unknown-linux-musl"
  "aarch64-unknown-linux-musl"
)

cd "$repo_root"

for target in "${targets[@]}"; do
  case "$target" in
    *linux-musl) cargo zigbuild --release --target "$target" ;;
    *) cargo build --release --target "$target" ;;
  esac

  case "$target" in
    x86_64-apple-darwin) label="macos-x86_64" ;;
    aarch64-apple-darwin) label="macos-arm64" ;;
    x86_64-unknown-linux-musl) label="linux-x86_64-musl" ;;
    aarch64-unknown-linux-musl) label="linux-arm64-musl" ;;
    *) label="$target" ;;
  esac

  package="mcp-doctor-${version}-${label}.tar.gz"
  tmpdir="$(mktemp -d)"
  cp "target/$target/release/mcp-doctor" "$tmpdir/mcp-doctor"
  cp README.md LICENSE "$tmpdir/"
  tar -C "$tmpdir" -czf "$dist/$package" .
  shasum -a 256 "$dist/$package" > "$dist/$package.sha256"
  rm -rf "$tmpdir"
  echo "wrote $dist/$package"
done
