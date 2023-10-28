# how to get started with keyscript:
- check out the keyscript website for repo, issues, docs, updates and more: `https://keyscript.org`
- go to the GitHub repo to install keyscript: `https://github.com/keyscript/keyscript`
- install the keyscript version that matches your machine: `https://github.com/keyscript/keyscript/releases/tag/v0.1.0`
- go to the directory of the installed release.

## for windows:
- run `./keyscript.exe init` to generate the starter files.
- run `./keyscript.exe ./file.kys` to compile a keyscript file.
- run `./keyscript.exe ./file.kys debug` to compile the file and generate a readable .wat file.
- run `./keyscript.exe ./file.kys gen` to compile the file and generate the necessary js code to import the functions automatically.
- you can also add gen after debug.

## for linux and mac:
- run `./keyscript init` to generate the starter files.
- run `./keyscript ./file.kys` to compile a keyscript file.
- run `./keyscript ./file.kys debug` to compile the file and generate a readable .wat file.
- run `./keyscript ./file.kys gen` to compile the file and generate the necessary js code to import the functions automatically.
- you can also add gen after debug.

## using keyscript with html:
- to use keyscript, you need to host the html file containing the js code.
- when running `./keyscript init`, you will also generate the `index.html` file.
- or when running `./keyscript ./file.kys gen`, you will generate the `file.html` containing the necessary js code for importing all the keyscript functions you created.
- you can use the vscode extension `live server` to host the html file easily.