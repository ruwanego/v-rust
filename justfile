default: ci

set windows-shell := ["pwsh.exe", "-NoProfile", "-Command"]

# Refresh the GitNexus knowledge graph after structural changes.
reindex:
    pnpm --allow-build=@ladybugdb/core --allow-build=gitnexus --allow-build=tree-sitter dlx gitnexus@latest analyze

fmt:
    cargo fmt --all -- --check

# The PR gate is LLVM-free: codegen_llvm is excluded here and covered by
# the weekly llvm-parity lane in progress.yml.
check:
    cargo check --locked --workspace --exclude codegen_llvm --all-targets

lint:
    cargo clippy --locked --workspace --exclude codegen_llvm --all-targets -- -D warnings

# LLVM-free inner loop for machines that cannot build inkwell/LLVM locally.
# Covers lexer/parser/sema red-green; codegen and fixtures run in CI.
unit-frontend *args:
    cargo test --locked --package frontend --lib {{args}}
    cargo test --locked --package codegen_traits --lib {{args}}

unit *args:
    cargo test --locked --package frontend --lib {{args}}
    cargo test --locked --package codegen_traits --lib {{args}}
    cargo test --locked --package codegen_cranelift {{args}}
    cargo test --locked --package v-rust --lib --bins {{args}}

tiny *args:
    cargo test --locked --package v-rust --test tiny_v_fixtures {{args}}

official-subset *args:
    cargo test --locked --package v-rust --test official_subset {{args}}

vlib-subset *args:
    cargo test --locked --package v-rust --test vlib_subset {{args}}

green: unit tiny official-subset vlib-subset

test: green

# LLVM backend parity lane. Needs a system LLVM 15; runs weekly in CI.
llvm-parity *args:
    cargo clippy --locked --package codegen_llvm --all-targets -- -D warnings
    cargo test --locked --package v-rust --no-default-features --features llvm --test tiny_v_fixtures {{args}}

vlib-full *args:
    cargo test --locked --package v-rust --test vlib_suite {{args}}

official-full *args:
    cargo test --locked --package v-rust --test official_suite {{args}}

vlib-progress *args:
    -cargo test --locked --package v-rust --test vlib_suite {{args}}

official-progress *args:
    -cargo test --locked --package v-rust --test official_suite {{args}}

pr-fast: fmt check lint green

heavy-progress: vlib-progress official-progress

merge-queue-heavy: pr-fast heavy-progress

ci: pr-fast
