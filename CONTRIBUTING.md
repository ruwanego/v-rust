# Contributing

This repository uses strict trunk-based development.

## Branches

1. Branch from `main`.
2. Keep branches short-lived: one day preferred, two days maximum.
3. Rebase on `origin/main`; do not merge `main` into a branch.
4. Do not push merge commits.
5. Expect GitHub squash-and-merge so the trunk receives one atomic commit per
   pull request.

## Required Guardrails

Use the `justfile` recipes. The project-level CI gate is:

```text
just pr-fast
```

This expands to:

```text
fmt -> check -> lint -> unit -> tiny -> official-subset -> vlib-subset
```

Do not rely on local success as proof. GitHub CI is the source of truth for pull
requests.

## TDD Policy

Compiler work should move in small red/green/refactor steps:

1. Add one failing Rust unit test or tiny V fixture.
2. Run it locally with `just` and confirm it is red for the expected reason.
3. Implement the smallest compiler change and return to green with `just ci`.
4. Push once per completed feature loop; confirm GitHub CI is green before merging.
5. Refactor only after the gate is green, and re-run `just ci`.

Full vlib and full official suites are currently expected-red progress logs.
Promote individual tests into subset manifests only when the same behavior is
already green through unit and tiny fixture coverage.
