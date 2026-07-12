# Strict TDD Roadmap

This project uses a layered TDD workflow for compiler construction. The official V
suite is the acceptance corpus, not the first red test for every change.

## Source Of Truth

Use the official V documentation as the language contract:

- Testing semantics: <https://docs.vlang.io/testing.html>
- Variables and assignment: <https://docs.vlang.io/variables.html>
- Functions: <https://docs.vlang.io/functions.html>
- Modules: <https://docs.vlang.io/modules.html>
- Primitive types, strings, numbers, arrays, maps: <https://docs.vlang.io/v-types.html>
- Statements and expressions: <https://docs.vlang.io/statements-%26-expressions.html>
- Structs and methods: <https://docs.vlang.io/structs.html>
- Type declarations, interfaces, sum types, options/results: <https://docs.vlang.io/type-declarations.html>

Do not add behavior from memory. Start every feature by locating the relevant
official doc section and recording the exact grammar or semantic rule in the
test name, fixture name, or test comment.

## Architecture Mapping Gate

Before adding a language feature, identify its Rust semantic home in
`ARCHITECTURE_MAPPING.md`.

Rules:

1. Parser-only syntax may stop at parser AST.
2. Anything with type, ownership, visibility, module, mutability, or runtime
   behavior must extend checked frontend output before backend code changes.
3. Parser AST nodes that can produce diagnostics must carry source spans.
4. Do not add backend special cases for semantics the frontend can prove.
5. If the mapping is unclear, update `ARCHITECTURE_MAPPING.md` first, then write
   the red test.

## Harness Layers

### L0: Rust Unit Tests

Purpose: isolate compiler internals before end-to-end noise.

Required only for layers affected by the language rule:

1. Lexer test for new tokens or literal forms.
2. Parser test for a new AST shape.
3. Semantic analyzer tests for valid and invalid type/scope behavior.
4. Codegen test when behavior reaches executable output.

Work outward one layer at a time. Each affected layer gets its own red/green
step and returns to green before the next layer starts. Do not create redundant
lexer or parser tests for rules that do not change those layers.

Run in CI through:

```text
just unit
```

### L1: Tiny V Fixtures

Purpose: prove a complete compile/link/run path with a minimal V program.

Pass fixtures live in:

```text
tests/fixtures/tiny/pass/*.v
tests/fixtures/tiny/pass/*.stdout
```

Fail fixtures live in:

```text
tests/fixtures/tiny/fail/*.v
tests/fixtures/tiny/fail/*.stderr
```

Rules:

1. A pass fixture must compile to a binary.
2. The generated binary must execute successfully.
3. stdout must exactly equal the `.stdout` file.
4. A fail fixture must fail compilation.
5. stdout plus stderr must contain the `.stderr` substring.
6. Each fixture covers one behavior only.

Run in CI through:

```text
just tiny
```

### L2: Official Subset

Purpose: promote supported official V tests into the green gate one at a time.

The manifest is:

```text
tests/official_subset.txt
```

Rules:

1. Add a path only after L0 and L1 are green for the same behavior.
2. Paths are relative to `tests/v_official_repo`.
3. Each promoted path must pass under `v-rust test <path>`.
4. Do not add a path just because it happens to pass accidentally.
5. If a path covers several unsupported features, do not promote it yet.
6. Prefer the smallest official file that exercises the completed feature.

Run in CI through:

```text
just official-subset
```

### L3: Vlib Subset

Purpose: keep V standard-library tests visible as their own acceptance lane.

The manifest is:

```text
tests/vlib_subset.txt
```

Rules:

1. Add a path only after L0 and L1 are green for the same behavior.
2. Paths are relative to `tests/v_official_repo/vlib`.
3. Each promoted path must pass under `v-rust test <vlib/path>`.
4. Do not add a path just because it happens to pass accidentally.
5. Vlib subset tests are part of the green gate once promoted.

Run in CI through:

```text
just vlib-subset
```

### L4: Vlib Full Progress

Purpose: keep an isolated progress log for V's standard-library `_test.v` files.

Rules:

1. This layer is expected to fail until the compiler can handle real V modules.
2. It must not block the green gate yet.
3. When failures shrink, promote the smallest stable vlib path into
   `tests/vlib_subset.txt`.

Run in CI through:

```text
just vlib-full -- --nocapture
```

### L5: Full Official Suite Progress

Purpose: keep ordered progress logs against V's full `_test.v` corpus.

Rules:

1. This layer is expected to fail until the compiler is far more complete.
2. It must log ordered `RUN`, `PASS`, and `FAIL` lines.
3. It must not be the only red signal used for implementation.
4. CI runs it after the green gate as non-blocking progress telemetry.
5. When full-suite failures shrink, promote the smallest newly passing official
   tests into L2.

Run in CI through:

```text
just official-full -- --nocapture
```

### Green Gate

The blocking project gate is:

```text
just ci
```

That expands to:

```text
fmt -> lint -> unit -> tiny -> official-subset -> vlib-subset
```

The full vlib suite and full official suite are intentionally outside the green
gate until they are no longer expected to fail.

## Feature Micro-Loop

Use this exact loop for every compiler feature:

1. Select exactly one V semantic rule from the official docs.
2. Write down the doc page and section in the issue, PR, or commit notes.
3. Identify the Rust semantic home in `ARCHITECTURE_MAPPING.md`.
4. For each affected L0 layer in lexer/parser/sema/codegen order, add one
   focused failing test, run `just unit`, implement that layer, and return to
   green before continuing.
5. Once the lower layers are green, add one tiny V fixture for the same rule.
6. Run `just tiny` and verify the fixture fails at the remaining end-to-end
   behavior, not an already-completed lower layer.
7. Implement the smallest remaining change and return `just tiny` to green.
8. Run `just ci` locally and verify it is fully green.
9. Refactor only after green, then run `just ci` again.
10. Inspect the full official suite progress log.
11. Inspect the vlib progress log when the feature touches standard-library
    behavior.
12. If a relevant official test is now supported, add exactly one path to
    `tests/official_subset.txt`.
13. If a relevant vlib test is now supported, add exactly one path to
    `tests/vlib_subset.txt`.
14. Run `just ci` locally, push once, and verify GitHub CI is green before
    merging.
15. Leave the full vlib and full official suites running as non-blocking
    telemetry.

The local `just ci` gate is the red/green inner loop; it runs natively on any
machine because Cranelift is the default backend (no LLVM install). GitHub
Actions is the merge gate: a feature is done only when CI is green, but
pushing is never how you discover whether a test is red.

## Current Baseline

The compiler currently supports only a tiny subset:

1. `fn main() { ... }`
2. zero-argument function declarations
3. integer and string literals
4. boolean literals in the AST and sema
5. basic arithmetic and comparison expressions
6. local variable declarations with `:=`
7. mutable local assignment with `mut`
8. `println(...)` as a builtin
9. native binary generation through Cranelift and the platform linker
   (LLVM remains as an opt-in parity backend)
10. typed semantic output through `frontend::sema::CheckedProgram`
11. primitive frontend `Type` values instead of stringly semantic type names
12. byte spans on parser AST nodes used by semantic diagnostics
13. semantic error rendering with file, line, and column locations

The compiler does not yet fully support normal V test semantics. In official V,
`_test.v` files are compiled as separate programs, test function names start
with `test_`, and `testsuite_begin`/`testsuite_end` have special meaning. The
current `v-rust test` command discovers `_test.v` files and compiles/runs each
generated binary, but it does not yet synthesize or execute V test functions.

## Roadmap

### Phase 1: Make Single-File V Shape Valid

#### 1.1 Comments

1. Add lexer tests for line comments.
2. Add lexer tests for block comments if the docs-confirmed syntax is in scope.
3. Add a tiny pass fixture where comments appear before `fn main`.
4. Ensure comments produce no parser-visible tokens.
5. Run full-suite progress and record the first failure that moved.

#### 1.2 Module Declarations

1. Add `module` token.
2. Add parser support for optional `module name` at file start.
3. Store module name in `Program`.
4. Reject non-initial module declarations.
5. Add tiny pass fixture with `module main`.
6. Add tiny fail fixture with a misplaced module declaration.
7. Promote the smallest official test that only needed module parsing.

#### 1.3 Imports

1. Add `import` token.
2. Parse simple imports.
3. Parse selective imports only after simple imports are green.
4. Add AST nodes without resolving modules at first.
5. Add sema error for unresolved non-builtin imports.
6. Add tiny fail fixture for unresolved import.
7. Add tiny pass fixture only for imports the compiler can resolve.

#### 1.4 Zero-Argument Function Signatures And Return Types

1. Collect all zero-argument function signatures before analyzing bodies.
2. Resolve calls to functions declared before or after the caller.
3. Parse `fn name() Type`.
4. Add `return` token and statement.
5. Enforce return expression type.
6. Enforce missing return for non-void functions.
7. Allow `main` to omit a return type.
8. Generate returned values and zero-argument calls in both backends.
9. Add a tiny fixture calling a later-declared returning function.
10. Promote one isolated official function-order test when test-function
    synthesis can execute it.

#### 1.5 Function Parameters

1. Parse parameter list as `name type`.
2. Extend the 1.4 function signatures with parameter types.
3. Enforce arity.
4. Enforce argument types.
5. Bind parameters in function scope.
6. Generate backend function params (Cranelift first).
7. Add tiny pass fixture for `fn add(x int, y int) int`.
8. Add tiny fail fixture for wrong arity.

### Phase 2: Correct V Test Semantics

#### 2.1 Parse `assert`

1. Add `assert` token.
2. Parse `assert expression`.
3. Parse optional extra assertion message only after basic assert is green.
4. Sema requires expression type `bool`.
5. Codegen aborts or returns non-zero on failed assert.
6. Tiny pass fixture with true assert.
7. Tiny fail fixture with false assert.

#### 2.2 Discover Test Functions Inside `_test.v`

1. Parse function names starting with `test_`.
2. In `v-rust test`, compile `_test.v` files without requiring user `main`.
3. Synthesize a test entrypoint that calls each `test_` function.
4. Preserve deterministic source order inside the file.
5. Print test function level pass/fail output.
6. Add tiny test fixture with two test functions.
7. Add tiny fail fixture where the second test fails and order is visible.

#### 2.3 `testsuite_begin` And `testsuite_end`

1. Detect optional `testsuite_begin`.
2. Detect optional `testsuite_end`.
3. Run begin before all `test_` functions.
4. Run end after all `test_` functions when prior tests pass.
5. Decide and test failure behavior when begin fails.
6. Add official subset path only after behavior matches docs.

#### 2.4 Internal Test File Compilation

1. Support `module main` in test files.
2. Include same-directory non-test `.v` files for internal tests.
3. Keep each `_test.v` compiled as a separate program.
4. Ignore `testdata` during discovery.
5. Add fixtures with `hello.v` plus `hello_test.v`.
6. Promote a matching official internal test.

#### 2.5 External Test File Compilation

1. Parse imports used by external tests.
2. Compile imported public API only.
3. Reject private symbol access across modules.
4. Add fixtures with a tiny module plus external test.
5. Promote a matching official external test.

### Phase 3: Expressions And Statements

#### 3.1 Full Numeric Literal Coverage

1. Decimal ints.
2. Underscore separators.
3. Hex ints.
4. Binary ints.
5. Octal ints.
6. Float literals.
7. Explicit casts like `i64(123)`.
8. Type inference defaults: int for integer literals, f64 for float literals.

#### 3.2 String Semantics

1. Single-quoted strings.
2. Double-quoted strings if supported.
3. Escape sequences.
4. String concatenation with `+`.
5. Reject string plus int without conversion.
6. String interpolation as a separate feature.
7. `println(string)` output exactly matches V.

#### 3.3 Boolean Semantics

1. `true` and `false`.
2. `!`, `&&`, `||`.
3. Short-circuit behavior.
4. Comparison results.
5. `println(bool)` must match V output, not C integer output.

#### 3.4 `if` Statements

1. Parse `if cond {}`.
2. Require boolean condition.
3. Parse `else`.
4. Parse `else if`.
5. Generate branches.
6. Add pass and fail tiny fixtures.

#### 3.5 `if` Expressions

1. Parse expression form.
2. Require all branches produce compatible types.
3. Require `else` when used as an expression.
4. Generate phi or equivalent value.
5. Add fixture from docs shape.

#### 3.6 `for` Forms

1. Infinite `for {}`.
2. C-style `for init; cond; post {}` if supported.
3. Range `for i in 0 .. n {}`.
4. Array iteration after arrays exist.
5. `break`.
6. `continue`.
7. Labelled break/continue only after normal break/continue.

#### 3.7 `match`

1. Parse literal match branches.
2. Parse `else`.
3. Match as statement.
4. Match as expression.
5. Exhaustiveness for enums and sum types later.

### Phase 4: Types

#### 4.1 Type Representation

Status: pulled forward as a foundation gate.

1. Done: replace stringly typed sema results with a `Type` enum.
2. Done: return `CheckedProgram` from semantic analysis.
3. Done: make LLVM codegen consume checked frontend output.
4. Next: model signedness and integer widths instead of only `i64`.
5. Next: add floats and numeric literal defaulting.
6. Done: carry source spans from lexer/parser into semantic diagnostics.
7. Done: render semantic diagnostics with file, line, and column locations.
8. Next: ensure all type comparison remains structural or stable-ID based.

#### 4.2 Arrays

1. Parse array literals.
2. Infer element type from first element.
3. Reject mixed element types.
4. Parse indexing.
5. Bounds behavior as a runtime concern.
6. Parse push operator `<<`.
7. Add `len` and `cap` fields after field access exists.

#### 4.3 Maps

1. Parse map literals.
2. Enforce key/value types.
3. Parse index access.
4. Implement `in` for map keys.
5. Preserve ordered map behavior only when runtime representation exists.

#### 4.4 Structs

1. Parse struct declarations.
2. Parse fields.
3. Default private immutable fields.
4. Parse access sections.
5. Parse struct literals.
6. Sema field initialization.
7. Sema required fields.
8. Generate struct layout.
9. Generate field access.
10. Generate mutable field assignment only when allowed.

#### 4.5 Methods

1. Parse receiver syntax.
2. Bind methods to receiver type.
3. Enforce same-module receiver rule.
4. Desugar method call to function call internally.
5. Add mutable receiver support later.

### Phase 5: Modules And Projects

#### 5.1 File Collection

1. Compile a single file.
2. Compile a directory.
3. Include all `.v` files in a module.
4. Exclude `_test.v` outside test mode.
5. Exclude `testdata`.
6. Sort files deterministically.

#### 5.2 Module Rules

1. Ordinary module files start with `module folder_name`.
2. Top-level project folder exception.
3. Short snake_case module names as a lint or semantic diagnostic.
4. Reject circular imports.
5. Static linking model is semantic, not dynamic runtime behavior.

#### 5.3 Visibility

1. Private by default.
2. `pub` functions/types.
3. Private fields inaccessible outside module.
4. Public immutable fields readonly.
5. `pub mut` behavior.

### Phase 6: Advanced V

Do not begin these until earlier phases have green gates and official subset
coverage:

1. Enums.
2. Interfaces.
3. Sum types.
4. Option/result types.
5. Error propagation.
6. Generics.
7. Compile-time features.
8. Attributes.
9. C interop.
10. Unsafe blocks.

## Promotion Checklist

Before adding any official test to `tests/official_subset.txt`, verify:

1. The official file path is stable in the cloned repo.
2. The file does not depend on broad unsupported syntax.
3. A Rust unit test covers the core compiler rule.
4. A tiny fixture proves executable behavior or expected rejection.
5. `just ci` is green in GitHub Actions.
6. The full-suite progress log shows the target behavior is not hidden behind an
   earlier parser failure.
7. Only one official path is promoted per commit unless the paths are exact
   variants of the same feature.

## Failure Policy

Unexpected failures:

1. `fmt` failure means fix formatting only.
2. `lint` failure means fix the implementation shape before adding behavior.
3. `unit` failure means the compiler layer is not correct yet.
4. `tiny` failure means the end-to-end behavior is not correct yet.
5. `official-subset` failure means a promoted acceptance test regressed.
6. `official-full` failure is expected progress telemetry until explicitly
   changed in this document.

Expected red commits:

1. A red commit should introduce exactly one failing test for the current
   affected layer or one L1 fixture after lower layers are green.
2. Its commit message should identify the feature under construction.
3. It should not include unrelated refactors.
4. It should be followed by a green implementation commit.

Refactor commits:

1. Must start from green CI.
2. Must preserve green CI.
3. Must not add new language behavior.
4. Must not promote official tests.
