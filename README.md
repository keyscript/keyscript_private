# keyscript syntax

- keyscript starts from the first line
- keyscript types: boolean, int, float, string (arrays will be added in a later version).
- Variable declaration: Use `int identifier = value;` syntax, variables do not require an initial value.
- Control flow: keyscript uses the `<`, `>`, `<=`, `>=`, `==`, `&&`, `||` operators for control flow.
- Arithmetic operations: keyscript uses `+`, `-`, `*`, `/`, `%`, `+=`, `-=`, `*=`, `/=` for basic arithmetic operations.
- If statement:
  `if bool_expression {
  code
  } else {
  code
  }`
- Loops:
- While loop: `while bool_expression { code }`
- for loops will be added in a later version in rust style `for i in 0..10 { code }`
- import keyscript functions using javascript. check `cargo run init` for example.
- I/O: keyscript uses `print()` for output, use js for input.
- keyscript also allows string concatenation `"hi" + " " + "there"` would be `hi there`.
- functions: keyscript uses the return type with a fucntion name and (parameters) syntax.
- functions can either return: `bool`, `int`, `float`, `void` (no return type)
```
int add(int a, int b) {
  return a + b;
}
```

## possible future features

- arrays
- foreach loops
- generators
- switch statements
