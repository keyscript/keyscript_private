# keyscript syntax

- keyscript starts from the first line, that is keyscript's main function.
- keyscript's types: `boolean`, `int`, `float`, `string`
- Variable declaration: Use `int identifier = value;` syntax, variables do not require an initial value.
- Control flow: keyscript uses the `<`, `>`, `<=`, `>=`, `==`, `&&`, `||` operators for control flow.
- Arithmetic operations: keyscript uses `+`, `-`, `*`, `/`, `%`, `+=`, `-=`, `*=`, `/=` for basic arithmetic operations.
- If statement:
  `if boolean_expression {
  code
  } else {
  code
  }`
- Loops:
- While loop: `while boolean_expression { code }`
- I/O: keyscript uses `print()` for output, use js for input.
- keyscript also allows string concatenation `"hi" + " " + "there"` would be `hi there`.
- functions: keyscript uses the return type with a function name and (parameters) syntax.
- functions can either return: `bool`, `int`, `float`, `void` (no return type)
```
int add(int a, int b) {
  return a + b;
}
```