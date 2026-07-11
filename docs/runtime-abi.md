# Runtime ABI

This document records runtime-facing representations that both native backends
must share. It is intentionally narrower than the future runtime crate.

## Pinned Sources

The source of truth is V 0.5.1 at commit
`0c3183c55b39534f9bb0d2f796bb575d39c9d229`:

- `tests/v_official_repo/vlib/builtin/string.v`, `struct string`
- `tests/v_official_repo/vlib/builtin/printing.c.v`, `println`
- `tests/v_official_repo/doc/docs.md`, `Functions`

Live documentation URLs remain useful references, but the pinned files and
compiler behavior decide this repository's compatibility target.

## Entrypoint

The platform entrypoint is exported as `main` and returns a 32-bit process
status. A V `fn main()` without an explicit return type completes with status
zero. Other V return types use their checked frontend type; backends do not
infer source-language return semantics.

## Strings

The target V string representation contains:

1. a pointer to NUL-terminated bytes;
2. an `int` byte length that excludes the terminating NUL;
3. an `int is_lit` ownership marker, where `1` identifies static literal data.

Until the runtime crate lands, backends may carry static string literals as an
internal pointer/byte-length pair and treat `is_lit` as `1`. They must preserve
embedded percent characters and use length-aware output. Source string bytes
must never be passed as a C format string.

The current frontend maps its integer subset to `Type::I64`. That is a temporary
compiler representation, not a frozen V ABI decision; Phase 4 owns V integer
width and signedness compatibility.

## Runtime Milestone

Before Phase 3.2 expands string semantics, add a shared runtime crate that owns
the concrete string ABI, allocation/freeing, printing, panic/assert support,
and backend link inputs. Cranelift and LLVM must call the same runtime surface
instead of growing independent builtin implementations.
