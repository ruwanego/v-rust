default: ci

fmt:
    cargo fmt --all -- --check

lint:
    cargo clippy --locked --all-targets --all-features -- -D warnings

unit *args:
    cargo test --locked --lib --bins --all-features {{args}}

tiny *args:
    cargo test --locked --test tiny_v_fixtures --all-features {{args}}

official-subset *args:
    cargo test --locked --test official_subset --all-features {{args}}

green: unit tiny official-subset

test: green

official-full *args:
    cargo test --locked --test official_suite --all-features {{args}}

ci: fmt lint green
