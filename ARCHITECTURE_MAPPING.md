# Architecture Mapping

This document records how V language concepts map into this Rust compiler.
It is a design guardrail: when a new V feature does not have a clear Rust home
here, update this document before adding implementation code.

## Core Rule

Do not map V files to Rust files. Map V semantic responsibilities to Rust
ownership, type, trait, and module boundaries.

The compiler pipeline is:

```text
source text
  -> lexer tokens
  -> parser AST
  -> checked frontend program
  -> backend trait
  -> backend implementation
  -> artifact
```

Parser AST is syntax. `CheckedProgram` is the first backend-consumable semantic
form. Backends must not own V language rules that can be expressed in the
frontend.

Every parser AST node that can participate in semantic diagnostics carries a
byte `Span`. Semantic diagnostics must preserve those spans so CLI rendering can
produce file, line, and column locations.

## Ownership Mapping

V values map to owned Rust values by default.

V `mut` locals map to scoped Rust mutability in semantic state, not to globally
mutable structures.

V references, parent links, and graph edges should map to stable IDs such as
`ScopeId`, `ExprId`, `TypeId`, or `ModuleId` once the graph outgrows direct
ownership. Prefer arenas plus IDs over `Rc<RefCell<T>>`; use shared interior
mutability only when the ownership graph genuinely requires it.

V `nil`, `voidptr`, and `unsafe` patterns must not leak into normal compiler
code. Map them to `Option<T>`, typed handles, explicit FFI wrapper modules, or
structured unsupported-feature diagnostics.

## Type Mapping

The frontend owns a Rust `Type` representation. String names are display output,
not semantic identity.

Current primitive mapping:

```text
V void/string/bool/int-literal-subset -> Type::Void/String/Bool/I64
```

Future extensions should grow this into:

```text
Type
  Void
  Bool
  Int(width, signedness)
  Float(width)
  String
  Array(TypeId)
  Map(TypeId, TypeId)
  Struct(StructId)
  Enum(EnumId)
  Sum(SumId)
  Function(FnSigId)
  Option(TypeId)
  Result(TypeId, ErrorId)
```

V sum types should map to Rust enums or typed sum metadata, never string tags.
Matching and smart casts should be frontend type-checker behavior.

V `?T` maps to `Option<T>` when absence is the whole semantic payload. V `!T`
maps to `Result<T, E>` with a custom error enum for the layer. V `or {}` maps to
explicit Rust recovery, transformation, or propagation depending on the source
semantics.

## Trait Mapping

Use static dispatch for hot compiler traversals and lowering internals:

```text
fn walk<V: Visitor>(visitor: &mut V, node: NodeId)
```

Use dynamic dispatch at coarse runtime boundaries:

```text
Box<dyn Backend>
```

V interfaces should not automatically become `Box<dyn Trait>`. Choose
`impl Trait` or generics when the implementation is known at compile time; choose
trait objects only for heterogeneous storage or runtime backend selection.

## Concurrency Mapping

Do not add async runtime machinery just because V has `go`, `spawn`, channels,
or `shared`.

Compiler concurrency should prefer deterministic work partitioning:

```text
parse files independently -> send parsed results -> resolve/typecheck centrally
```

Use channels for ownership transfer between workers. Use `Arc<T>` for immutable
shared data. Use `Arc<Mutex<T>>` or `Arc<RwLock<T>>` only when multiple threads
must mutate the same state. Use atomics only for counters, flags, and lock-free
state with simple invariants.

## Module Mapping

Target crate responsibilities:

```text
crates/frontend
  source and diagnostics
  lexing and parsing
  checked frontend program
  name resolution
  type checking
  module graph and visibility

crates/codegen_traits
  backend selection
  backend trait
  backend diagnostics

crates/codegen_cranelift
  fast debug backend

crates/codegen_llvm
  optimized LLVM backend

crates/driver
  CLI
  test harness
  frontend/backend orchestration
```

Default visibility is private. Use `pub(crate)` for cross-module internals and
`pub` only for crate API surfaces that other crates must consume.

## Current Guardrail

The backend boundary is `frontend::sema::CheckedProgram`. If a feature needs new
syntax, first add parser AST with spans. If it needs meaning, add checked
frontend nodes and typed diagnostics before codegen learns about it.
