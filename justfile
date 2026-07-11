default: ci

# Refresh the GitNexus knowledge graph after structural changes.
reindex:
    pnpm --allow-build=@ladybugdb/core --allow-build=gitnexus --allow-build=tree-sitter dlx gitnexus@latest analyze

fmt:
    cargo fmt --all -- --check

check:
    cargo check --locked --workspace --all-targets --all-features

lint:
    cargo clippy --locked --workspace --all-targets --all-features -- -D warnings

# LLVM-free inner loop for machines that cannot build inkwell/LLVM locally.
# Covers lexer/parser/sema red-green; codegen and fixtures run in CI.
unit-frontend *args:
    cargo test --locked --package frontend --lib {{args}}
    cargo test --locked --package codegen_traits --lib {{args}}

unit *args:
    cargo test --locked --package frontend --lib --all-features {{args}}
    cargo test --locked --package codegen_traits --lib --all-features {{args}}
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
