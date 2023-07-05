use std::{fs::read_to_string, path::Path, process::Command};

use inkwell::{
    context::Context,
    passes::PassManager,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};
use pest::Parser;

use crate::{code_generator::Compiler, parser::parse_program};

pub mod code_generator;
pub mod internal_representations;
pub mod parser;
pub mod state_machine;

fn generate_object_code(code: String, name: &str) {
    let ast = parser::Parser::parse(parser::Rule::program, &code)
        .unwrap()
        .next()
        .unwrap();
    let program = parse_program(ast).unwrap();

    Target::initialize_x86(&InitializationConfig::default());

    let target_triple = TargetMachine::get_default_triple();
    println!("{:#?}", target_triple);
    let target = Target::from_triple(&target_triple).unwrap();

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "x86-64",
            "+avx2",
            OptimizationLevel::Default,
            RelocMode::Default,
            CodeModel::Default,
        )
        .unwrap();

    let context = Context::create();
    let module = context.create_module("main");
    let builder = context.create_builder();

    module.set_data_layout(&target_machine.get_target_data().get_data_layout());
    module.set_triple(&target_triple);

    let fpm = PassManager::create(&module);

    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();
    fpm.add_gvn_pass();
    fpm.add_cfg_simplification_pass();
    fpm.add_basic_alias_analysis_pass();
    fpm.add_promote_memory_to_register_pass();
    fpm.add_instruction_combining_pass();
    fpm.add_reassociate_pass();

    fpm.initialize();

    let _function: inkwell::values::FunctionValue =
        Compiler::compile(&context, &builder, &fpm, &module, &program).unwrap();

    target_machine
        .write_to_file(&module, FileType::Object, Path::new(&format!("{}.o", name)))
        .unwrap();

    target_machine
        .write_to_file(
            &module,
            FileType::Assembly,
            Path::new(&format!("{}.as", name)),
        )
        .unwrap();
}

fn generate_executable(name: &str) {
    let mut command = Command::new("clang++-12");
    command.arg(format!("{}.o", name));
    command.arg("-o");
    command.arg(name);
    command.output();
}

fn main() {
    // get file name
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        println!("Please provide a file name");
        return;
    }

    let file_name = &args[1];

    let code = read_to_string(file_name).unwrap();

    let name = file_name.strip_suffix(".ac").unwrap();

    generate_object_code(code, name);

    generate_executable(name);
}
