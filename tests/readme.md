# End to end tests of `ono`
This binary hosts and runs end-to-end tests on the ono interpreter.
Tests are located in `.ono-test` files and follow this format:
```
let a = 1;
let b = 2;
let c = a + b;
print(f"{c}");
--ERR--
--OUT--
3
```
A test might also test for one or more errors. This is done like so:
```
--IN--
let a = "foo";
let b = 1;
let c = a + b;
--ERR--
type error: cannot 'string + number'
-> temp.ono 3:11
3 | let c = a + b;
              ^
--OUT--
```
