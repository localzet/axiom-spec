# axiom-spec v0.2.0

Компилятор языка намерений/спецификаций Axiom 0.2. В DSL v0.2 появились именованные proof obligations и неограниченные
целочисленные области.

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

Выходной формат — канонический interchange `AXIOM-IR/2`.
