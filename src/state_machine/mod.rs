use std::{ffi::OsStr, path::Path};

use anyhow::bail;
use inkwell::{
    context::Context,
    passes::PassManager,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    OptimizationLevel,
};
use pest::Parser;
use walkdir::WalkDir;

use crate::{code_generator::Compiler, parser};

use self::{
    data::{CompilerData, Config},
    file_tree::FileTree,
};

pub mod data;
pub mod file_tree;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    Init,
    FilesLoading,
    Parsing,
    LlvmIrGeneration,
    Linking,
    Done,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Event {
    LoadFiles,
    Parse,
    LlvmIrGen,
    Link,
}

pub struct CompilerMachine {
    state: State,
    data: CompilerData,
}

impl CompilerMachine {
    pub fn new(config: Config) -> Self {
        // TODO: run it only once
        Target::initialize_x86(&InitializationConfig::default());

        CompilerMachine {
            state: State::Init,
            data: CompilerData::new(config),
        }
    }

    pub fn accept(&mut self, event: Event) -> anyhow::Result<()> {
        match (self.state, event) {
            (State::Init, Event::LoadFiles) => {
                self.state = State::FilesLoading;
                let tree = self.load_files()?;
                println!("{}", tree.build_program());
            }
            (State::FilesLoading, Event::Parse) => {
                self.state = State::Parsing;
                self.parse()?;
            }
            (State::Parsing, Event::LlvmIrGen) => {
                self.state = State::LlvmIrGeneration;
                self.llvm_ir_gen()?;
            }
            (State::LlvmIrGeneration, Event::Link) => {
                self.state = State::Linking;
                self.linking()?;
                self.state = State::Done;
            }
            _ => bail!("Invalid state transition"),
        }

        Ok(())
    }

    fn load_files(&mut self) -> anyhow::Result<FileTree> {
        let source_dir = Path::new(&self.data.config.project_directory).join("src");

        let mut tree = FileTree::new(source_dir.clone());

        for entry in WalkDir::new(source_dir.clone()) {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                continue;
            }

            if path.extension() == Some(OsStr::new("ac")) {
                tree.insert(path, source_dir.as_path())
            }
        }

        println!("{:#?}", tree);
        Ok(tree)
    }

    fn parse(&mut self) -> anyhow::Result<()> {
        let raw_ast =
            parser::Parser::parse(parser::Rule::program, self.data.code.as_ref().unwrap())
                .unwrap()
                .next()
                .unwrap();
        self.data.ast = Some(parser::parse_program(raw_ast).unwrap());

        Ok(())
    }

    fn llvm_ir_gen(&mut self) -> anyhow::Result<()> {
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

        target_machine
            .write_to_file(&module, FileType::Object, Path::new("out.o"))
            .unwrap();

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

        let function: inkwell::values::FunctionValue = Compiler::compile(
            &context,
            &builder,
            &fpm,
            &module,
            self.data.ast.as_ref().unwrap(),
        )
        .unwrap();

        Ok(())
    }

    pub fn linking(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
