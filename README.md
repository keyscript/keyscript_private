# keyscript syntax

- keyscript starts from a main function
- keyscript types: boolean, int, float, string, and array (vectors maybe later).
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
- For loops are c styled: `for (int i = 0; i < 10; i++) { code }`
- While loop: `while bool { code }`
- Scoping: keyscript uses `{}` for scoping.
# stopped
- module system and importing:   
- I/O: keyscript uses `print()` for output and `std::read()` for input.
- klang allows custom format inside strings `"hi {1+2}"` would be `hi 3`, but dont allow recursive formatting (formatting inside formatting).
- meaning you can print anything you want using 1 print statement! for example: `print("3 pi is: {3 * std::pi()}");`
- Error handling: Klang does not feature explicit error handling. Errors are handled by the parser, scanner, and compiler, and reported to the developer in the terminal.
- Functions: All functions in Klang are public.
- the way you declare a function is: `fn name(arg1, arg2) {`
- you can then use return value; or return; to quit the function and return a value.
- Example:
```klang
fn add(int1, int2) {
    return int1 + int2;
}
print("3 + 5 = {add(3, 5)}");
```
- klang offers a veriety of native functions, each runs in rust! here are the native functions klang offers:
- Math Functions: `sin` `cos` `tan` `sqrt` `pow` `ln` `log` `round` `abs` `min` `max` `pi`
- Random Functions: `random` `range` `randbool`
- Time Functions: `time` `sleep`
- File I/O Functions: `readFile` `writeFile` `read`
- Vector functions: `get(vec, index)` `set(vec, value, index)` `remove(vec, index)` `insert(vec, value, index)`
- use them by doing `std::` and add the function name
