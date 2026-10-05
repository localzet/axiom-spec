# axiom-spec v0.2.0

Compiler for the Axiom 0.2 intent/specification language. The v0.2 DSL introduces named proof obligations and unbounded
integer domains.

```text
axiom 0.2
module abs
input x int
output result int
domain x unbounded
requires true
ensures nonnegative: result >= 0
ensures magnitude: result == x || result == -x
objective instructions min
```

Output is the canonical interchange format `AXIOM-IR/2`.

## Авторство

Сопровождающий собственных изменений: **Ivan Zorin (localzet)** — <creator@localzet.com> · https://www.localzet.com. Copyright © 2026 Localzet Group. Исходное авторство и лицензии сторонних компонентов сохраняются. См. [AUTHORS](.github/AUTHORS.md).
