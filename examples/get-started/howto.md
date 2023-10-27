to get started and verify functionality (check that the language is working):  

- change to an appropriate directory for working within your system  
`cd ~/*`  

- clone the keyscript language repository  
(using HTTP) `git clone https://github.com/keyscript/keyscript`  
(using SSH) `git clone git@github.com:keyscript/keyscript`  

- change directory to location of the cloned repository  
`cd ~/*/keyscript`  

- generate working starter files ["index.html", "index.kys", "index.wasm"]  
`cargo run init`  

- optional: generate a readable webassembly (.wat) file using the `debug` after your command
`cargo run ./index.kys debug`  

- host the index.html file *(webpage should read `Function returned: 102334155` aka fib(40))*  
    - option a: using an extension such as 'Live Server' in VSCode  
        1. right-click index.html file in explorer pane  
        2. 'Open with Live Server [Alt+L Alt+O]'

<br>

***
<br>

to write new code using keyscript (after the steps listed above are completed):  

- change the contents of index.kys by writing your own code such as `print("kys");`  

- recompile with the contents (cargo run ./index.kys)  

- optional: generate a **new** readable webassembly (.wat) file to reflect any changes made  
`cargo run ./index.kys debug`

- optional #2: generate the necessary js code to import the functions automatically using `gen` at the end:
- `cargo run ./index.kys gen` / `cargo run ./index.kys debug gen`

- within index.html find the line containing the following code (line 48)  
`const returnValue = result.instance.exports.fib(BigInt(40));`  

- change "fib" to "main" and remove "BigInt(40)" to make the line as follows  
`const returnValue = result.instance.exports.main();`
- or to any function you created in your keyscript file. `result.instance.exports.func(params);`

- also make sure to check out `primes.kys` for a more complex example of keyscript syntax.
