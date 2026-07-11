# v-rust — V Language Compiler in Rust

Goal: implement the entire V language (<https://docs.vlang.io>) in Rust, one
semantic rule at a time, driven by layered TDD against the official V test
corpus.

## Binding Workflow — read these before any code change

1. `AGENT.md` — non-negotiable rules, pre-flight checklist, feature micro-loop.
2. `docs/tdd-roadmap.md` — current phase/step, harness layers L0–L5.
3. `ARCHITECTURE_MAPPING.md` — the Rust semantic home for every V concept.
4. `docs/repository-strategy.md` — workspace layout, migration order.

Summary of the loop (full version in `AGENT.md`):

- One V semantic rule per feature, with the official doc URL recorded.
- Red L0 unit test → red L1 tiny fixture → smallest implementation →
  local `just ci` green → refactor → promote at most one official/vlib test
  path → push once → confirm GitHub CI green with `gh`.
- Local testing: `just ci` runs natively on any machine — Cranelift is the
  default backend, no LLVM install needed. The LLVM lane is weekly CI only.
- Never implement behavior from memory; the docs and the pinned official V
  repo (`tests/v_official_repo`) are the language contract.

## Precedence

`AGENT.md` and the docs above are binding. The auto-generated GitNexus
section below is an optional exploration aid: its "Always Do"/"Never Do"
rules (mandatory impact analysis, detect_changes before commit) do NOT
apply — the compiler changes daily and the index is routinely stale. Use
GitNexus tools only when they genuinely speed up code navigation.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **v-rust** (495 symbols, 1036 relationships, 41 execution flows). Use the GitNexus MCP tools to understand code, assess impact, and navigate safely.

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
