#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
cd "$root"
if [[ ! -f Cargo.toml || -d cli-rust ]]; then
  echo 'Release from a standalone goodissues_cli clone after syncing the subtree.' >&2
  exit 1
fi
if [[ -n "$(git status --porcelain)" ]]; then
  echo 'Commit all changes before releasing.' >&2
  exit 1
fi
tag=${1:?Usage: scripts/publish.sh vX.Y.Z}
version=$(cargo metadata --no-deps --format-version 1 | python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')
if [[ "$tag" != "v$version" ]]; then
  echo "Tag must match Cargo.toml: v$version" >&2
  exit 1
fi
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
git tag "$tag"
git push origin "refs/tags/$tag"
