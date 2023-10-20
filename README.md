# keyscript syntax

- keyscript starts from a main function
- keyscript types: boolean, int, float, char, string, and array (vectors maybe later).
- Variable declaration: Use `int identifier = value;` syntax, variables do not require an initial value.
- Control flow: keyscript uses the `<`, `>`, `<=`, `>=`, `==`, `&&`, `||` operators for control flow.
- Arithmetic operations: keyscript uses `+`, `-`, `*`, `/`, `%` for basic arithmetic operations.
- If statement:
  `if expression {
  code
  } else {
  code
  }`
- Loops:
- While loop: `while bool { code }`
- for loops will be added later in rust style `for i in 0..10 { code }`
- Scoping: keyscript uses `{}` for scoping.
- module system and importing: `use {function} from "./file.kys";`
- I/O: keyscript uses `print()` for output and `unknown std function` for input.
- keyscript allows custom format inside strings `"hi {1+2}"` would be `hi 3`
- keyscript also allows string concatenation `"hi" + " " + "there"` would be `hi there`.
- Error handling: `prob a later feature`
- functions: keyscript uses the return type with a fucntion name and (parameters) syntax.
- you can also declare a function public by using `pub` before the function name.
```
pub int add(int a, int b) {
  return a + b;
}
```
- you can call a function with named variables like `add(a=1, b=2)`

## possible future features

- classes
- vectors
- error handling
- foreach loops
- generators
- switch statements
- multithreading
- async
