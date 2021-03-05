## Example

```
[
  Atom("hello"),
  Nest(
    "(",
    Group(
      "this",
      Space,
      "is",
      Space,
      "inside"
    ),
    ")"
  )
]
```

Wide output (width >= 40):

```
hello(this is inside)
```

Next breakpoint (width >= 17):

```
hello(
  this is inside
)
```

Final breakpoint:

```
hello(
  this
  is
  inside
)
```

IR:

```
start:
Bounded("hello")
Bounded("(")
Break(1)
IncreaseNesting
Bounded("this")
Break(2)
Bounded("is")
Break(2)
Bounded("inside")
DecreaseNesting
Break(1)
Bounded(")")
```

### Len (width=40, break=0)

```
Bounded("hello")
=> break=0, cursor=5, nesting=0
Bounded("(")
=> break=0, cursor=6, nesting=0
Break(1)
=> break=0, cursor=6, nesting=1
IncreaseNesting
=> skip
Bounded("this")
=> break=0, cursor=10, nesting=1
Break(2)
=> skip
Bounded("inside")
=> break=0, cursor=16, nesting=1
DecreaseNesting
=> break=0 cursor=16, nesting=0
Break(2)
=> skip
Bounded(")")
=> break=0, cursor=17, nesting=0
=> break=0
```

### Len (width=16, break=0)

```
start:
Bounded("hello")
=> break=0, cursor=5, nesting=0
Bounded("(")
=> break=0, cursor=6, nesting=0
Break(1)
=> break=0, cursor=6, nesting=1
IncreaseNesting
=> skip
Bounded("this")
=> break=0, cursor=10, nesting=1
Break(2)
=> skip
Bounded("inside")
=> break=0, cursor=16, nesting=1
Break(2)
=> skip
DecreaseNesting
=> break=0 cursor=16, nesting=0
Bounded(")")
=> break=0, cursor=17, nesting=0
=> ❌
=> backtrack to start(break=1)

Bounded("hello")
=> break=1, cursor=5, nesting=0
Bounded("(")
=> break=1, cursor=6, nesting=0
Break(1)
=> newline
=> break=1, cursor=0, nesting=0
IncreaseNesting
=> break=1, cursor=2, nesting=1
Bounded("this")
=> break=1, cursor=6, nesting=1
Break(2)
=> skip
Bounded("inside")
=> break=0, cursor=16, nesting=1
Break(2)
=> skip
DecreaseNesting
=> break=0 cursor=16, nesting=0
Bounded(")")
=> break=0, cursor=17, nesting=0

```
