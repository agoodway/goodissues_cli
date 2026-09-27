# Run from either the monorepo subtree or a standalone clone.
build:
    cargo build --locked

release:
    cargo build --release --locked

test:
    cargo test --locked

check:
    cargo fmt --check
    cargo clippy --locked --all-targets -- -D warnings
    cargo test --locked

run *ARGS:
    cargo run -- {{ARGS}}

# Export committed Rust changes from the monorepo to the standalone repository.
sync:
    bash scripts/sync-subtree.sh

# Release from a standalone clone after updating Cargo.toml and Cargo.lock.
# Pushing a tag triggers all six builds and publication in GitHub Actions.
publish tag:
    bash scripts/publish.sh {{quote(tag)}}
