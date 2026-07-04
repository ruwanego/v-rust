# Agent Guardrails

This file is the mandatory pre-flight for any LLM-driven change to this
repository. Read it completely before writing any code.

## Non-Negotiable Rules

1. **Testing never happens locally.** Do not run `cargo test`, `just unit`,
   `just tiny`, or any test command in the local shell. The local environment
   is not the source of truth. CI is.

2. **CI is the only oracle.** After every push, check CI with `gh`. A green
   local build means nothing. A green CI run means the change is correct.

3. **One feature per branch per PR.** No combining language features,
   refactors, and migration steps in one commit.

4. **Read the roadmap before picking work.** Do not implement a feature that
   is not in the current phase. Do not skip phases.

5. **Update `ARCHITECTURE_MAPPING.md` before writing implementation code.**
   If a new feature has no clear Rust home in that file, add one first. The
   mapping update and the implementation must not be in the same commit.

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

Next: **1.4 Function Return Types**

Do not start 1.5 or later until 1.4 is promoted through L0 and L1.

## Feature Micro-Loop

Follow this exactly. Do not compress or reorder steps.

1. Identify the single V semantic rule. Record the official doc URL and
   section.
2. Confirm the feature is in the current phase of `docs/tdd-roadmap.md`.
3. Identify or add the Rust semantic home in `ARCHITECTURE_MAPPING.md`.
4. Write one failing Rust unit test (L0). Do not implement yet.
5. Push. Check CI:
   ```
   gh run list --limit 5
   gh run view <run-id> --log-failed
   ```
   CI must be red at the expected test. If CI is green, the test is wrong.
6. Write one failing tiny V fixture (L1). Do not implement yet.
7. Push. Check CI. CI must be red at the expected fixture.
8. Write the smallest implementation that satisfies both failures.
9. Push. Check CI. `just ci` must be fully green.
   ```
   gh run list --limit 5
   gh run view <run-id>
   ```
10. Refactor only after green. Push. Verify CI stays green.
11. Inspect full-suite progress:
    ```
    gh run list --workflow=progress.yml --limit 3
    ```
12. If a relevant official test now passes, add exactly one path to
    `tests/official_subset.txt`. Push. Verify CI green.

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
- CI is green before the red test was pushed (the test is not actually
  failing — something is wrong).
- The implementation spans more than one V semantic rule.
- A refactor is mixed into a feature commit.

## Promotion Checklist

Before adding a path to `tests/official_subset.txt` or
`tests/vlib_subset.txt`:

- [ ] L0 unit test covers the core compiler rule.
- [ ] L1 tiny fixture proves executable behavior or expected rejection.
- [ ] `just ci` is green in GitHub Actions (verified with `gh`).
- [ ] The official file does not depend on unsupported syntax.
- [ ] Only one path is promoted per commit.
