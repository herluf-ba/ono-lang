# Roadmap
- [x] Expressions

Get expressions working. Do some cleanup!

- [x] Type checking pass

Do typechecking of expressions

- [x] Tuple expressions

Implement tuples. Eliminate the null type in favor of unit tuple.

- [] End to end tests

create test suite that reads source files and compares to expected output

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

- [] Block expressions

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

- [] Tuple indexing

Implement a way to access the elements of a tuple
```rust
let t = (1, 2);
let sum_t = t.0 + t.1; // -> 3
```

- [] Tuple unpacking

Implement of "unpacking" a tuple into variables. Only 

```rust
let t = (1, 2, 3, 4, 5);
let a, b, c, d, e = t; // a = 1, b = 2, c = 3, d = 4, e = 5
let f, g, _ = t; // f = 1, g = 2 rest is unused.
let a, b, c, d, e, f = t; // type error: t is length 5 but 6 elements extracted  
```

- [] If-expressions

Implement if-expressions. Each branch must have the same return type.
If expressions must have an else branch.

# ⚠️ Disclaimer ⚠️

This is a hobby project. I cannot recommend depending on it for anything serious.
However, feel free to clone, modify and reuse any part of it if it brings you any value.
What follows is pure fiction.

## ono

`ono` is a typed scripting language inspired by rust and typescript.
It features optionals, tuples, iterators, enums, pattern matching and traits.

This should help you get a feel for the language:

```rust
// imports are done using a 'use' statement. 
// This one makes everything in './my_module.ono' available via 'my_module::{something}';
use my_module;

// This one brings everything from './my_other_module.ono' into scope
use my_other_module::*;

// This one only brings 'Enum' and 'Obj' into scope
use my_other_other_module::{Enum, Obj};

// primitive types are Number, Bool, String
// any type can be made optional using 'type?'. This wraps the value in 'Some(value)' or 'None'.
// any type can made into an iterable list using '[type]'
// all values are truthy except 'false', none and empty list.

// enums can be used to described variants
enum Species {
  Dog,
  Cat,
  Crocodile
}

// custom types are defined using object definitions
obj Animal {
  species: Species,
  name: String,
  nickname: String?,
  quirks: [String]
}

// traits are used to generalize behavior
trait Speak {
  // traits define a number of functions that make up the trait
  fn speak(self): String;
}

// objects implement traits using a 'make' statement
make Animal Speak {
  fn speak(self): String {
    // if the last line of a block is an expression
    // that expression is automatically returned
    // Also formatted strings work like this:
    f"Hi my name is {self.name}"
  }
}

// Automatic type conversions can be made using the into trait
trait Into<T> {
  fn into(self) -> T;
}

// operator implementations can be made using traits too
trait Add<T> {
  fn add(self, other: T) -> T;
}

// 'main' is treated as entrypoint 
fn main() {
  // objects can be constructed like this.
  // The type of variables is infered if possible
  let harold = Animal { name: "Harold", species: Species::Crocodile };
  harold.speak();

  // match expressions 
  let formatted_nickname = match harold.nickname {
    Some(nickname) => nickname,
    None => ""
  };

  // 'or' can provide a fallback for an optional
  let cooler_format = harold.nickname or "";

  // optionals can be used for control flow using 'if let'
  if let Some(nickname) = harold.nickname {
    // use nickname here
  }

  // a for loop may iterate any iterable. 
  // ranges can be constructed using 'start..end' or 'start..=end' for inclusive end
  for num in 0..harold.quirks.len() {
    // iterables can be indexed using 'iter[index]';
    let quirk = harold.quirks[num];
  }

  // Empty iterables are falsey
  if harold.quirks {
    for quirk in harold.quirks {
      print(f"I have got {quirk}");
    }
  }
}
```

