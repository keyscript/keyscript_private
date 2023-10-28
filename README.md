# keyscript syntax

- Keyscript starts from the first line, that is Keyscript's main function.
- Keyscript's types: `bool`, `int`, `float`, `string`
- Variable declaration: Use `int identifier = value;` syntax, variables do not require an initial value.
- Control flow: Keyscript uses the `<`, `>`, `<=`, `>=`, `==`, `&&`, `||` operators for control flow.
- Arithmetic operations: Keyscript uses `+`, `-`, `*`, `/`, `%`, `+=`, `-=`, `*=`, `/=` for basic arithmetic operations.
- If statement:
  `if boolean_expression {
  code
  } else {
  code
  }`
- Loops:
- While loop: `while boolean_expression { code }`
- I/O: keyscript uses `print()` for output, use JS for input.
- Keyscript also allows string concatenation `"hi" + " " + "there"` would be `hi there`.
- Functions: Keyscript uses the return type with a function name and (parameters) syntax.
- Functions can either return: `bool`, `int`, `float`, `void` (no return type)
- Example of Keyscript's syntax:
```C
int add(int a, int b) {
  return a + b;
}
```


## possible future features

- arrays
- foreach loops
- generators
- switch statements
