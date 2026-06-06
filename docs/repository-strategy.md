# Repository Strategy

This repository is a Rust compiler project. The project target is a Cargo
workspace with a backend-neutral frontend and swappable code generation
backends. This document is binding project policy; when implementation and this
document disagree, either fix the implementation or update this document in the
same pull request.

## Target Workspace Architecture

The target workspace layout is:

```text
crates/
  frontend/
  codegen_traits/
  codegen_cranelift/
  codegen_llvm/
  driver/
```

### `frontend`

Owns the V source language:

1. Lexer.
2. Parser.
3. AST.
4. Semantic analyzer.
5. Type checker.
6. Frontend diagnostics.

Rules:

1. No backend imports.
2. No LLVM, Inkwell, Cranelift, object-file, or linker knowledge.
3. Public API returns typed frontend IR plus diagnostics.
4. Tests are ordinary Rust unit tests and tiny V fixtures that do not depend on
   a production backend.

### `codegen_traits`

Defines the abstract backend contract.

Initial scope:

1. Backend selection enum used by the driver.
2. Trait for lowering a checked frontend program.
3. Trait method for emitting object code or an executable artifact.
4. Structured backend diagnostics.
5. Optional text dump hook for IR snapshot tests.

No backend implementation crates may depend on each other.

### `codegen_cranelift`

Implements `codegen_traits` with Cranelift.

Purpose:

1. Fast debug builds.
2. Pull request feedback path.
3. End-to-end tiny fixtures.
4. First backend for new language features unless LLVM-specific behavior is the
   feature under test.

### `codegen_llvm`

Implements `codegen_traits` with LLVM through Inkwell or `llvm-sys`.

Purpose:

1. Optimized production builds.
2. Heavy merge queue validation.
3. LLVM IR snapshot testing.
4. Backend parity checks against Cranelift for already-supported frontend
   behavior.

### `driver`

Owns the `v-rust` CLI binary.

Responsibilities:

1. Parse CLI flags.
2. Select backend.
3. Orchestrate `frontend -> codegen_traits -> backend -> linker`.
4. Run the V test harness.
5. Print ordered progress logs.

Rules:

1. Driver can know about all backend crates.
2. Driver cannot own frontend semantics.
3. Driver cannot hide backend failures behind generic messages.

## Current State

The repository is currently a Cargo workspace with:

1. Frontend modules in `crates/frontend/src/lex`,
   `crates/frontend/src/parse`, and `crates/frontend/src/sema`.
2. LLVM/Inkwell code generation still in the root package at `src/codegen`.
3. CLI and test harness modules still in the root package at `src/driver`.

The migration to the target workspace must continue as short, reviewable trunk
branches. Do not combine the frontend split, backend trait extraction,
Cranelift backend, LLVM backend extraction, and snapshot testing in one pull
request.

## Migration Order

1. Done: move `src/lex`, `src/parse`, and `src/sema` into
   `crates/frontend`.
2. Next: add `crates/codegen_traits` with a minimal backend trait.
3. Move the current Inkwell implementation into `crates/codegen_llvm`.
4. Move `src/driver` and `src/main.rs` into `crates/driver`.
5. Add `crates/codegen_cranelift` with the smallest executable backend.
6. Change PR fast-path tests to use Cranelift only.
7. Add IR snapshot tests with `insta`.
8. Promote LLVM optimized checks to the merge queue heavy path.

Each migration PR must preserve the TDD guardrail shape:

```text
fmt -> check -> lint -> unit -> tiny -> official-subset -> vlib-subset
```

## Branching Strategy

Development model: strict trunk-based development.

Rules:

1. Branch from `main`.
2. Keep feature branches short-lived: target less than one day, hard cap two
   days.
3. Do not merge `main` into a feature branch.
4. Sync with upstream using `git rebase origin/main`.
5. Do not push merge commits.
6. Pull requests are squash-merged.
7. One pull request lands as exactly one atomic trunk commit.
8. Branch names should be short and descriptive, for example
   `frontend-crate-split`, `cranelift-min-backend`, or `ir-snapshots`.

Current long-running experimental branches should be treated as temporary
staging branches. Once they are reviewable, squash-merge them or replace them
with smaller branches cut from `main`.

## CI Strategy

### Pull Request Fast Path

Trigger: `pull_request` and branch pushes.

Target behavior:

1. `cargo fmt --all -- --check`.
2. `cargo check`.
3. `cargo clippy`.
4. Rust unit and integration tests.
5. Tiny V fixtures.
6. Official subset.
7. Vlib subset.
8. Cranelift backend only once Cranelift exists.

Until `codegen_cranelift` exists, the fast path keeps using the current backend
so the repository remains protected by executable fixtures.

### Merge Queue Heavy Path

Trigger: GitHub Merge Queue through `merge_group`.

Target behavior:

1. Build LLVM/Inkwell using runner-provided LLVM packages.
2. Run optimized LLVM backend checks.
3. Run end-to-end regression suites.
4. Run full official and vlib progress suites.
5. Fail only after supported subsets are promoted out of telemetry.

The full official and vlib suites are currently expected-red telemetry. They
must stay visible in CI logs, but they must not block trunk until the compiler
has a meaningful supported subset.

## IR Snapshot Testing

Use `insta` for backend IR snapshots after backend crates are split.

Snapshot targets:

1. Cranelift CLIF for fast backend tests.
2. LLVM IR for optimized backend tests.
3. Tiny fixtures that exercise one semantic behavior per snapshot.

Rules:

1. Snapshot files are checked into the repository.
2. Snapshot updates must be reviewed in pull request diffs.
3. Snapshot churn without a matching source-language behavior change is a
   regression risk.
4. Frontend tests should prefer AST or typed-IR snapshots; backend tests should
   snapshot backend IR.

Do not add broad snapshot fixtures. Add one focused fixture per language
feature.
