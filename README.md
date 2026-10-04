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

## Связанные исследования

Этот компонент входит в исследовательский проект [Axiom](https://github.com/localzet/axiom-stack). Все компоненты собраны по теме [localzet-axiom](https://github.com/topics/localzet-axiom). Основной язык документации — русский. Исследовательские результаты и ограничения не означают готовность к промышленному применению.
