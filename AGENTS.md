# v-rust — V Language Compiler in Rust

Goal: implement the entire V language (<https://docs.vlang.io>) in Rust, one
semantic rule at a time, driven by layered TDD against the official V test
corpus.

This file is the mandatory pre-flight for any LLM-driven change to this
repository. Read it completely before writing any code, together with:

1. `docs/tdd-roadmap.md` — current phase/step, harness layers L0–L5.
2. `ARCHITECTURE_MAPPING.md` — the Rust semantic home for every V concept.
3. `docs/repository-strategy.md` — workspace layout, migration order,
   branching rules.

## Non-Negotiable Rules

1. **Local `just ci` is the inner loop, and it needs no LLVM.** Cranelift is
   the default backend, so the full gate — including tiny fixtures that
   compile, link, and run real binaries — runs on any machine with Rust and
   a platform linker (MSVC on Windows, cc on Unix). Run it at every
   red/green step. Never push to CI just to find out whether a test is red
   or green. The LLVM backend lane (`just llvm-parity`) runs weekly in CI
   and is not part of the inner loop.

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

7. **No doc drift.** If a change makes any statement in `AGENTS.md`,
   `ARCHITECTURE_MAPPING.md`, or `docs/` false, fix that statement in the
   same commit. `AGENTS.md` is the single canonical entry-point for all
   coding agents; `CLAUDE.md` must contain only the `@AGENTS.md` import
   line (plus any auto-generated GitNexus block) — never duplicate content
   into it. Before ending a work session, re-read the "Current Phase"
   section below and the Migration Order in `docs/repository-strategy.md`
   and update them to match reality.

## Pre-Flight Checklist

Before touching any `.rs` or `.toml` file, answer these in order:

- [ ] What is the current phase and step in `docs/tdd-roadmap.md`?
- [ ] Is this feature listed under that phase? If not, stop.
- [ ] What is the Rust semantic home in `ARCHITECTURE_MAPPING.md`?
- [ ] Does `ARCHITECTURE_MAPPING.md` need updating first?
- [ ] What is the single V semantic rule from the official docs being
      implemented? (URL and section required.)

## Current Phase

**Next: 1.4 Function Return Types**

The backend migration (steps 2–6 of the Migration Order in
`docs/repository-strategy.md`) is complete: Cranelift is the default
backend and the full gate runs locally without LLVM. Do not start 1.5 or
later until 1.4 is promoted through L0 and L1.

## Feature Micro-Loop

Follow this exactly. Do not compress or reorder steps. All red/green checks
run locally with `just <recipe>`; no LLVM or Docker is needed.

1. Identify the single V semantic rule. Record the official doc URL and
   section in the commit message or PR description.
2. Confirm the feature is in the current phase of `docs/tdd-roadmap.md`.
3. Identify or add the Rust semantic home in `ARCHITECTURE_MAPPING.md`.
4. Write one failing Rust unit test (L0). Run `just unit` — it must fail at
   exactly that test. If it passes, the test is wrong.
5. Write one failing tiny V fixture (L1). Run `just tiny` — it must fail at
   exactly that fixture.
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

The full official and vlib suites and the LLVM parity lane are non-blocking
telemetry. They run weekly. Do not treat their failures as blockers.

## Hard Stops

Stop immediately and do not proceed if any of the following is true:

- The feature is not in the current phase of `docs/tdd-roadmap.md`.
- `ARCHITECTURE_MAPPING.md` has no home for the feature and has not been
  updated.
- The new L0 test or L1 fixture passes before the implementation exists (the
  test is not actually testing the new behavior — something is wrong).
- The implementation spans more than one V semantic rule.
- A refactor is mixed into a feature commit.
- A binding doc still describes the pre-change behavior after your commit
  (see rule 7: fix the doc in the same commit).

## Promotion Checklist

Before adding a path to `tests/official_subset.txt` or
`tests/vlib_subset.txt`:

- [ ] L0 unit test covers the core compiler rule.
- [ ] L1 tiny fixture proves executable behavior or expected rejection.
- [ ] `just ci` is green locally, then in GitHub Actions (verified with `gh`).
- [ ] The official file does not depend on unsupported syntax.
- [ ] Only one path is promoted per commit.

## Precedence

This file and the docs listed at the top are binding. The auto-generated
GitNexus section below is an optional exploration aid: its "Always Do"/
"Never Do" rules (mandatory impact analysis, detect_changes before commit)
do NOT apply — the compiler changes daily and the index is routinely stale.
Use GitNexus tools only when they genuinely speed up code navigation.
<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **v-rust** (493 symbols, 1036 relationships, 41 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

> Index stale? Run `node .gitnexus/run.cjs analyze` from the project root — it auto-selects an available runner. No `.gitnexus/run.cjs` yet? `npx gitnexus analyze` (npm 11 crash → `npm i -g gitnexus`; #1939).

## Always Do

- **MUST run impact analysis before editing any symbol.** Before modifying a function, class, or method, run `impact({target: "symbolName", direction: "upstream"})` and report the blast radius (direct callers, affected processes, risk level) to the user.
- **MUST run `detect_changes()` before committing** to verify your changes only affect expected symbols and execution flows. For regression review, compare against the default branch: `detect_changes({scope: "compare", base_ref: "main"})`.
- **MUST warn the user** if impact analysis returns HIGH or CRITICAL risk before proceeding with edits.
- When exploring unfamiliar code, use `query({search_query: "concept"})` to find execution flows instead of grepping. It returns process-grouped results ranked by relevance.
- When you need full context on a specific symbol — callers, callees, which execution flows it participates in — use `context({name: "symbolName"})`.
- For security review, `explain({target: "fileOrSymbol"})` lists taint findings (source→sink flows; needs `analyze --pdg`).

## Never Do

- NEVER edit a function, class, or method without first running `impact` on it.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit changes without running `detect_changes()` to check affected scope.

## Resources

| Resource | Use for |
|----------|---------|
| `gitnexus://repo/v-rust/context` | Codebase overview, check index freshness |
| `gitnexus://repo/v-rust/clusters` | All functional areas |
| `gitnexus://repo/v-rust/processes` | All execution flows |
| `gitnexus://repo/v-rust/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
|------|---------------------|
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
