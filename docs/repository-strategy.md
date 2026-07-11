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
2. Frontend semantic analysis returns typed checked output for backends.
3. Backend contract in `crates/codegen_traits`; Cranelift (default backend)
   in `crates/codegen_cranelift`; LLVM/Inkwell (opt-in `llvm` feature) in
   `crates/codegen_llvm`.
4. CLI, pipeline orchestration, and test harness in the root `v-rust`
   package, which serves as the driver crate.

Remaining migration work must continue as short, reviewable trunk branches,
one step per pull request.

## Migration Order

This migration is prioritized ahead of further language features (see
`AGENTS.md` Current Phase). Goal: the default local build and the full
`just ci` gate must not require a system LLVM install. Cranelift is a
pure-Rust dependency; LLVM stays behind an optional cargo feature used by
the merge-queue heavy path.

1. Done: move `src/lex`, `src/parse`, and `src/sema` into
   `crates/frontend`.
2. Done: add `crates/codegen_traits` with a minimal backend trait; the
   root-package Inkwell path implements it.
3. Done: move the current Inkwell implementation into `crates/codegen_llvm`
   behind the root `codegen` feature; the root package no longer depends on
   Inkwell directly.
4. Done (by decision, not by move): the root `v-rust` package is the driver
   crate. After steps 2–3 it contains only the CLI, the pipeline
   orchestration in `src/compiler.rs`, the test runner, and the test
   harness that resolves the `v-rust` binary. Physically relocating it to
   `crates/driver` is deferred until it blocks something; revisit after
   step 8.
5. Done: add `crates/codegen_cranelift` with the smallest executable backend.
   Emits object code with `cranelift-object`; links with the platform linker
   (MSVC `link.exe` on Windows, `cc` on Unix), not clang. Covered by an
   end-to-end crate test that compiles, links, and runs a V program.
6. Done: Cranelift is the default backend feature and the PR fast path is
   LLVM-free (`--workspace --exclude codegen_llvm`; building or testing
   without `--features llvm` does not compile Inkwell). The LLVM backend is
   exercised by the weekly `llvm-parity` lane in progress.yml.
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
8. Cranelift backend only; `codegen_llvm` is excluded from the fast path and
   covered by the weekly `llvm-parity` lane.

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
