use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction,
    Module, TypeSection, ValType,
};
use std::fs;

pub fn wasm() {
    let mut module = Module::new();

    let mut types = TypeSection::new();
    let params = vec![ValType::I32, ValType::I32];
    let results = vec![ValType::I32];
    types.function(params, results);
    module.section(&types);

    // Encode the function section.
    let mut functions = FunctionSection::new();
    let type_index = 0;
    functions.function(type_index);
    module.section(&functions);

    // Encode the export section with the function named "add".
    let mut exports = ExportSection::new();
    exports.export("add", ExportKind::Func, 0);
    module.section(&exports);

    // Encode the code section.
    let mut codes = CodeSection::new();
    let locals = vec![];
    let mut f = Function::new(locals);
    f.instruction(&Instruction::LocalGet(0));
    f.instruction(&Instruction::LocalGet(1));
    f.instruction(&Instruction::I32Add);
    f.instruction(&Instruction::End);
    codes.function(&f);
    module.section(&codes);


    //output:
    let wasm_bytes = module.finish();
    fs::write("output.wasm", &wasm_bytes).expect("Failed to write Wasm to file");
    fs::write("output.wat", wasmprinter::print_file("./output.wasm").unwrap()).expect("Failed to write Wat to file");
}
