
### The endless todo

- [ ] End to end tests (blocked until we have `main` entrypoint)

create test suite that reads source files and compares to expected output.
Extend `onoi` to return the value of the last expression. Statements will return `()`.
Then test all language features and all error productions.

- [ ] Add comments to the grammar
Comments are ignored by the lexer right now. 
This can be problematic as they can appear in the middle of another structure.
```
while 
# This here is a comment
  a < 10 {
 a = a + 1;
};
```
Adding comments to the grammar restricts where they can appear, making it easier to reason about transformations (think desugaring or auto formatting).

- [ ] Refactor error messages
Error messages could be more flexible.
They should contain a `Vec<SrcArea>` each with line start and count along with highlight column start and count.
That way it is possible to underline any series of square blocks of src code enabling scenarios like:
```
[E000] error: expected bool found number
-> main.ono 3:7
1  let a = 1;
2
3  while a + 1 {
         ^^^^^
```

```
[E000] error: expected branches to have same type but found bool and ()
-> main.ono 3:7
1  if true {
       ...
10     a > 2
       ^^^^^
11 } else {
12     a < 2;
       ^^^^^
```
It might be worthwhile to implement a `impl From<Expr> for SrcArea` to help spawn errors.

- [x] While loops
Implement good old while loops
```rust
while a < b {
  a = a + 1;
}
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
let (a, b, c, d, e) = t; // a = 1, b = 2, c = 3, d = 4, e = 5
let (f, g, ...) = t; // f = 1, g = 2 rest is unused.
let (a, b, c, d, e, f) = t; // type error: t is length 5 but 6 elements extracted  
```

- [x] Block expressions

Implement a rust style block that returns the value of the last expression in it. 
Blocks must end in an expression
```rust
let a = 3
let c = {
  let b = 1 + 2;
  a + b
};
// c -> 6
```

- [x] If-expressions

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
although the trailing ';' is uncool

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
