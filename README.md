# ⚠️ Disclaimer ⚠️

This is a hobby project. I cannot recommend depending on it for anything serious.
However, feel free to clone, modify and reuse any part of it if it brings you any value.

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

## Roadmap

### Feat: Expressions

Get expressions working. Do some cleanup!

todo: 
-[x] Make Token::new() take start column instead of end. Refactor lexer and parser
-[ ] Remove bang in favor of not 


### Feat: Type checking pass

Do typechecking of expressions

### Feat: Tuple expressions

Implement tuples. Eliminate the null type in favor of unit tuple.

### Test: Block expressions?

Try to implement a rust style block that returns a value if the last thing in it is an expression.
useful for if statements, functions, and local variable declarations.
Maybe empty blocks can return unit type and be treated as expression statements?

