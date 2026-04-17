# Archive Policy

## Active code
- `src/lib.rs`
- `src/recumbent.rs`
- `src/wassan.rs`

## Archived regression code
- `src/cram_ns.rs`
- Any symbol prefixed `archived_`

## Rule
Archived code may remain in-tree for fast regression and arithmetic pressure testing,
but it must not drive active theorem planning, public API design, or 3D proof claims.

## Naming discipline
Every archived helper should keep the `archived_` prefix. If a symbol does not carry
that prefix, an agent may reasonably assume it is active.
