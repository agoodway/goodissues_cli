#!/usr/bin/env bash
set -euo pipefail
root=$(git rev-parse --show-toplevel)
if [[ ! -f "$root/cli-rust/Cargo.toml" ]]; then
  echo 'Run sync from the GoodIssues monorepo.' >&2
  exit 1
fi
cd "$root"
if [[ -n "$(git status --porcelain -- cli-rust)" ]]; then
  echo 'Commit the Rust CLI changes before syncing.' >&2
  exit 1
fi
git subtree push --prefix=cli-rust git@github.com:agoodway/goodissues_cli.git main
