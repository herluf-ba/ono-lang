
# Roadmap
- [ ] End to end tests

create test suite that reads source files and compares to expected output.
Extend `onoi` to return the value of the last expression. Statements will return `()`.
Then test all language features and all error productions.


- [ ] Block expressions

Implement a rust style block that returns the value of the last expression in it.
Implicitly return () if the there is no such expression
```rust
let a = 3
let c = {
  let b = 1 + 2;
  a + b
};
// c -> 6

let d = {
  let x = true;
};
// d -> ()
```

- [ ] Tuple indexing

Implement a way to access the elements of a tuple
```rust
let t = (1, 2);
let sum_t = t.0 + t.1; // -> 3
```

- [ ] Tuple unpacking

Implement of "unpacking" a tuple into variables. Only 

```rust
let t = (1, 2, 3, 4, 5);
let a, b, c, d, e = t; // a = 1, b = 2, c = 3, d = 4, e = 5
let f, g, _ = t; // f = 1, g = 2 rest is unused.
let a, b, c, d, e, f = t; // type error: t is length 5 but 6 elements extracted  
```

- [ ] If-expressions

Implement if-expressions. Each branch must have the same return type.
If expressions must have an else branch.
```rust
let a = if 1 >= 2 { "foo" } else { "bar" };
```

A side effect is that regular if statements are now possible as an expression statement
```rust
if 1 > 2 {
  let foo = true;
} else {
  let bar = false;
}; // An expression of type ()
```
although the trailing ';' is uncool, and the else branch is mandatory.

- [x] Expressions

Get expressions working. Do some cleanup!

- [x] Type checking pass

Do typechecking of expressions

- [x] Tuple expressions

Implement tuples. Eliminate the null type in favor of unit tuple.

- [x] expression statements

If an expression is followed by a semicolon, it marks that the result is not used.
The expression should merely be evaluated for its side-effect.
This is useful for declarations, but also later for stateful function calls.
```rust
  1 + 2; // <- typechecked and evaluated but then ignored.
```

- [x] Declaration statements

Implement declarations to a single global scope. Declarations take the form
```rust
  let a: number = 1;
  let b = 1; // type infered from initializer
  let c: bool = 1 + 2; // result in type error
```

- [x] Variable expressions

Implement a way to refer to variables in scope.
```rust
let a = 1;
let b = a * 2; // evaluates to 2
```

- [x] Assignment expressions
Implement assignments.
```rust
let a = 1;
a = 2;
a = true; // <- type error
```
