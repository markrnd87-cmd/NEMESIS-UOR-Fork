# uor-foundation-macros

Compile-time DSL for the UOR term language.

This crate provides the `uor!` procedural macro that parses EBNF surface
syntax at compile time and produces typed `Term` ASTs in the
`uor_foundation::enforcement` module.

**This crate is re-exported by `uor-foundation`.** Add `uor-foundation`
as your dependency — do not depend on this crate directly.

```toml
[dependencies]
uor-foundation = "0.1"
```

## Grammar Reference

| Form | Syntax | Produces |
|------|--------|----------|
| Term expression | `add(mul(3, 5), 7)` | `TermArena` with the expression tree |
| Quantum literal | `42@Q7` | `Term::Literal` at the specified level |
| Type declaration | `type Pixel { residue: 255; }` | `TypeDeclaration` |
| Binding | `let x : Pixel = add(0, 0);` | `Binding` with surface syntax and content address |
| Assertion | `assert add(1, 2) = 3;` | Compile-time check (ground) or `Assertion` struct |
| Effect | `effect Blit { target: {0,1}; delta: 0; commutes: true; }` | Effect metadata |
| Source | `source name : Type via grounding;` | `SourceDeclaration` |
| Sink | `sink name : Type via projection;` | `SinkDeclaration` |
| Lift | `lift(x, Q3)` | `Term::Lift` (lossless level injection) |
| Project | `project(y, Q0)` | `Term::Project` (lossy level surjection) |
| Match | `match x { pred => expr; otherwise => expr; }` | Pattern-match term |
| Recursion | `recurse f(n) measure n base p => e step => e` | Bounded recursive term |
| Stream | `unfold nat : Successor from 0` | Coinductive stream constructor |

## Examples

### Type Declaration

Defines a constrained type with residue and hamming constraints:

```rust,ignore
use uor_foundation::uor;

let pixel = uor! {
    type Pixel {
        residue: 255;
        hamming: 8;
        depth: 1;
    }
};
```

### Operation Composition

Builds a term expression tree bottom-up in a `TermArena`:

```rust,ignore
use uor_foundation::uor;

let sum = uor! { add(mul(3, 5), 7) };
// The surface syntax "add(mul(3, 5), 7)" is embedded as a &'static str.
// The content address is computed at compile time via FNV-1a.
```

### Ground Assertion

Ground assertions (no free variables) are checked at **compile time**.
If the ring arithmetic does not hold, compilation fails:

```rust,ignore
use uor_foundation::uor;

// These pass — ring arithmetic in Z/256Z:
uor! { assert add(1, 2) = 3; };
uor! { assert mul(3, 5) = 15; };

// This would FAIL at compile time:
// uor! { assert add(1, 2) = 4; };
```

### Boundary Declaration

Declares data sources and sinks at the kernel/external boundary:

```rust,ignore
use uor_foundation::uor;

uor! { source pixel_in  : Pixel via sRGB; };
uor! { sink   pixel_out : Pixel via DisplayP3; };
```

## Error Behavior

Parse errors produce `compile_error!()` at the macro call site:

```text
uor! { add(1, ) }
// error: uor! parse error: Unexpected token in expression: RParen
```

Ground assertions that fail ring evaluation also produce compile errors:

```text
uor! { assert add(1, 2) = 4; }
// error: assertion failed at compile time
```

## License

Apache-2.0
