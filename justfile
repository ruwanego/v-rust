default: ci

fmt:
    cargo fmt --all -- --check

lint:
    cargo clippy --locked --all-targets --all-features -- -D warnings

test *args:
    cargo test --locked --all-targets --all-features {{args}}

ci: fmt lint test
