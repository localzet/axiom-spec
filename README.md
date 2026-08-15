# axiom-spec

The front door of Axiom: a deliberately small intent/specification language that compiles human-editable contracts into
deterministic `AXIOM-IR/1`.

> **Maturity:** research prototype v0.1. The default verifier proves properties by exhaustive evaluation over an
> explicitly finite input domain. A VALID receipt is therefore a theorem about that bounded model, not a claim of
> unbounded program correctness.

## Example

```text
axiom 0.1
module abs
input x i64
output result i64
domain x -16 16
requires true
ensures result >= 0
ensures result == x || result == -x
objective instructions min
```

```bash
cargo run -- compile examples/abs.ax --out abs.aix
```

The compiler canonicalizes whitespace and ordering so downstream components can hash the same logical artifact
reproducibly.
