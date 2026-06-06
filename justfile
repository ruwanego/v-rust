default: ci

fmt:
    cargo fmt --all -- --check

check:
    cargo check --locked --workspace --all-targets --all-features

lint:
    cargo clippy --locked --workspace --all-targets --all-features -- -D warnings

unit *args:
    cargo test --locked --package frontend --lib --all-features {{args}}
    cargo test --locked --package v-rust --lib --bins --all-features {{args}}

tiny *args:
    cargo test --locked --package v-rust --test tiny_v_fixtures --all-features {{args}}

official-subset *args:
    cargo test --locked --package v-rust --test official_subset --all-features {{args}}

vlib-subset *args:
    cargo test --locked --package v-rust --test vlib_subset --all-features {{args}}

green: unit tiny official-subset vlib-subset

test: green

vlib-full *args:
    cargo test --locked --package v-rust --test vlib_suite --all-features {{args}}

official-full *args:
    cargo test --locked --package v-rust --test official_suite --all-features {{args}}

vlib-progress *args:
    -cargo test --locked --package v-rust --test vlib_suite --all-features {{args}}

official-progress *args:
    -cargo test --locked --package v-rust --test official_suite --all-features {{args}}

pr-fast: fmt check lint green

heavy-progress: vlib-progress official-progress

merge-queue-heavy: pr-fast heavy-progress

ci: pr-fast
