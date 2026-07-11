# Agent Guardrails

This file is the mandatory pre-flight for any LLM-driven change to this
repository. Read it completely before writing any code.

## Non-Negotiable Rules

1. **Local red/green first, at whatever depth the machine allows.** Never
   push to CI just to find out whether a test is red or green when a local
   check can answer it. Depth ladder, use the deepest rung available:
   - Full: `just ci` (needs LLVM 15; or `docker compose run --rm app just ci`).
   - LLVM-free: `just unit-frontend` — lexer/parser/sema tests with no
     inkwell/LLVM build. On slow machines this is the required minimum for
     every red and green step that touches the frontend.
   - Codegen changes and tiny fixtures that cannot build locally are the only
     things allowed to get their first verification from GitHub CI.

2. **GitHub CI is the merge gate, not the test runner.** Push once per
   completed feature loop, then confirm CI is green with `gh` before merging.
   A feature is done only when GitHub CI is green.

3. **One feature per branch per PR.** No combining language features,
   refactors, and migration steps in one commit.

4. **Read the roadmap before picking work.** Do not implement a feature that
   is not in the current phase. Do not skip phases.

5. **Update `ARCHITECTURE_MAPPING.md` before writing implementation code.**
   If a new feature has no clear Rust home in that file, add one first. The
   mapping update and the implementation must not be in the same commit.

6. **Behavior comes from the official V docs and the pinned V corpus, never
   from memory.** Every feature records the official doc URL and section.
   When docs are ambiguous, the behavior of the pinned official `v` compiler
   (release tag in `tests/v_repo_ref.txt`) is the tie-breaker.

## Pre-Flight Checklist

Before touching any `.rs` or `.toml` file, answer these in order:

- [ ] What is the current phase and step in `docs/tdd-roadmap.md`?
- [ ] Is this feature listed under that phase? If not, stop.
- [ ] What is the Rust semantic home in `ARCHITECTURE_MAPPING.md`?
- [ ] Does `ARCHITECTURE_MAPPING.md` need updating first?
- [ ] What is the single V semantic rule from the official docs being
      implemented? (URL and section required.)

## Files To Read First

Every session, before any code change:

1. `docs/tdd-roadmap.md` — current phase, current step, feature micro-loop
2. `ARCHITECTURE_MAPPING.md` — where each concern lives
3. `docs/repository-strategy.md` — migration order, branching rules

## Current Phase

**Phase 1 — Make Single-File V Shape Valid**

Completed:
- 1.1 Comments
- 1.2 Module declarations
- 1.3 Imports (builtin allowlist resolution; selective imports parse-only)

Next: **Backend migration to Cranelift** — steps 2–6 of the Migration Order
in `docs/repository-strategy.md` (codegen_traits, codegen_llvm extraction,
driver crate, codegen_cranelift, Cranelift as the default test backend).

This is prioritized ahead of 1.4 so the full gate, including tiny fixtures,
runs locally without any LLVM install. One migration step per PR. Language
work resumes at **1.4 Function Return Types** once `just ci` passes locally
on a machine with no LLVM.

## Feature Micro-Loop

Follow this exactly. Do not compress or reorder steps. All red/green checks
run locally (natively or via `docker compose run --rm app just <recipe>`).

1. Identify the single V semantic rule. Record the official doc URL and
   section in the commit message or PR description.
2. Confirm the feature is in the current phase of `docs/tdd-roadmap.md`.
3. Identify or add the Rust semantic home in `ARCHITECTURE_MAPPING.md`.
4. Write one failing Rust unit test (L0). Run `just unit` (or
   `just unit-frontend` on machines without LLVM) — it must fail at exactly
   that test. If it passes, the test is wrong.
5. Write one failing tiny V fixture (L1). Run `just tiny` — it must fail at
   exactly that fixture. If `just tiny` cannot build locally (no LLVM), CI
   may provide this one verification.
6. Write the smallest implementation that satisfies both failures.
7. Run `just ci` locally. It must be fully green.
8. Refactor only after green. Run `just ci` again.
9. If a relevant official or vlib test is now supported, add exactly one path
   to `tests/official_subset.txt` or `tests/vlib_subset.txt` and re-run
   `just ci`.
10. Commit (feature and promotion may be separate commits) and push once.
11. Verify GitHub CI is green before merging:
    ```
    gh run list --limit 5
    gh run watch <run-id>
    gh run view <run-id> --log-failed
    ```

## CI Commands

Check current run status:
```
gh run list --limit 5
```

Watch a run in progress:
```
gh run watch <run-id>
```

View failure logs:
```
gh run view <run-id> --log-failed
```

Check a PR's checks:
```
gh pr checks <pr-number>
```

View the progress workflow (non-blocking, runs weekly):
```
gh run list --workflow=progress.yml --limit 3
```

## What CI Runs

The blocking gate (`just ci`) expands to:

```
fmt -> check -> lint -> unit -> tiny -> official-subset -> vlib-subset
```

The full official and vlib suites are non-blocking progress telemetry. They
run weekly. Do not treat their failures as blockers.

## Hard Stops

Stop immediately and do not proceed if any of the following is true:

- The feature is not in the current phase of `docs/tdd-roadmap.md`.
- `ARCHITECTURE_MAPPING.md` has no home for the feature and has not been
  updated.
- The new L0 test or L1 fixture passes before the implementation exists (the
  test is not actually testing the new behavior — something is wrong).
- The implementation spans more than one V semantic rule.
- A refactor is mixed into a feature commit.

## Promotion Checklist

Before adding a path to `tests/official_subset.txt` or
`tests/vlib_subset.txt`:

- [ ] L0 unit test covers the core compiler rule.
- [ ] L1 tiny fixture proves executable behavior or expected rejection.
- [ ] `just ci` is green locally, then in GitHub Actions (verified with `gh`).
- [ ] The official file does not depend on unsupported syntax.
- [ ] Only one path is promoted per commit.
