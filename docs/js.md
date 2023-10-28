# Using Your KeyScript Code with JavaScript
Keyscript is a language that compiles to WebAssembly. To use your keyscript code, you need to import the functions into JavaScript and use them there.

## Importing KeyScript Functions into JavaScript
There are three ways to import KeyScript functions into JavaScript:
1. Using the `./keyscript ./file.kys gen` Command:
    - This command generates a `file.html` file containing the necessary JavaScript code for importing all the Keyscript functions you created.

2. Importing Using a JavaScript Promise:
   - First, make sure to include the following JavaScript code:
   ```javascript
   let imports = {
        wasm: {
            memory: new WebAssembly.Memory({initial: 256}), // 1 page = 64KB, 256 pages = much storage
        },
        console: {
            log: function (offset, length) {
                console.log(new TextDecoder('utf8').decode(new Uint8Array(imports.wasm.memory.buffer, offset, length)));
            }
        }
   };
   ```
    - Then, use the following code to import the Keyscript functions:
    ```javascript
    fetch('index.wasm') // file name!!
        .then(response => response.arrayBuffer())
        .then(bytes => {
            return WebAssembly.instantiate(bytes, imports)
        })
        .then(result => {
            const returnValue = result.instance.exports.main(); // import functions here
            if (returnValue) {
                document.getElementById('output').textContent = `Function returned: ${returnValue}`;
            } else {
                document.getElementById('output').textContent = `No output, check the console`;
            }

        })
        .catch(error => {
            document.getElementById('error').textContent = `Error loading WebAssembly: ${error.message}`;
        })
    ```
   - The provided file also contains a bit of error handling and output representation.
   - To import a function, use `func = result.instance.exports.function_name`. you can now use `func` as a normal JavaScript function, by calling it with `func(params)`.
3. Importing inside a javascript async function:
   - First, make sure to include the following JavaScript code:
   ```javascript
   let imports = {
        wasm: {
            memory: new WebAssembly.Memory({initial: 256}), // 1 page = 64KB, 256 pages = much storage
        },
        console: {
            log: function (offset, length) {
                console.log(new TextDecoder('utf8').decode(new Uint8Array(imports.wasm.memory.buffer, offset, length)));
            }
        }
   };
   ```
    - Then, use the following code to import the Keyscript functions:
    ```javascript
    (async () => {
        const response = await fetch('index.wasm'); // file name!!
        const bytes = await response.arrayBuffer();
        const result = await WebAssembly.instantiate(bytes, imports);
        const main_func = result.instance.exports.main; // import functions here
    })();
    ```
   - To import a function, use `func = result.instance.exports.function_name`. you can now use `func` as a normal JavaScript function, by calling it with `func(params)`.
