# How to get started with keyscript:
- Check out the Keyscript website for repo, issues, docs, updates and more: `https://keyscript.org`
- Go to the GitHub repo to install keyscript: `https://github.com/keyscript/keyscript`
- Install the keyscript version that matches your machine: `https://github.com/keyscript/keyscript/releases/tag/v0.1.0`
- Go to the directory of the installed release.

## For windows:
- Run `./keyscript.exe init` to generate the starter files.
- Run `./keyscript.exe ./file.kys` to compile a Keyscript file.
- Run `./keyscript.exe ./file.kys debug` to compile the file and generate a readable .wat file.
- Run `./keyscript.exe ./file.kys gen` to compile the file and generate the necessary JS code to import the functions automatically.
- You can also add `gen` after `debug` for JS code generation.
  
## For linux and mac:
- Run `./keyscript init` to generate the starter files.
- Run `./keyscript ./file.kys` to compile a Keyscript file.
- Run `./keyscript ./file.kys debug` to compile the file and generate a readable .wat file.
- Run `./keyscript ./file.kys gen` to compile the file and generate the necessary JS code to import the functions automatically.
- You can also add `gen` after `debug` for JS code generation.

## Using Keyscript with html:
-To use Keyscript, you need to host the html file containing the JS code.
- When running `./keyscript init`, you will also generate the `index.html` file.
- Or when running `./keyscript ./file.kys gen`, you will generate the `file.html` containing the necessary JS code for importing all the keyscript functions you created.
- You can use the vscode extension `live server` to host the html file easily.
