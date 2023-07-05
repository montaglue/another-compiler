use std::collections::HashMap;

use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    passes::PassManager,
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, FunctionValue, IntValue, PointerValue},
    IntPredicate,
};

use crate::internal_representations::gast::{Expr, Function, Operator, Program, Stmt};

pub struct Compiler<'a, 'ctx> {
    pub context: &'ctx Context,
    pub builder: &'a Builder<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub module: &'a Module<'ctx>,

    variables: HashMap<String, PointerValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>,
}

impl<'a, 'ctx> Compiler<'a, 'ctx> {
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }

    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.fn_value().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.i64_type(), name)
    }

    fn compile_expr(&mut self, expr: &Expr) -> Option<IntValue<'ctx>> {
        match expr {
            Expr::IntLiteral(int) => Some(self.context.i64_type().const_int(*int as u64, true)),
            Expr::StringLiteral(_) => todo!(),
            Expr::Name(name) => Some(
                self.builder
                    .build_load(*self.variables.get(&name.name)?, name.name.as_str())
                    .into_int_value(),
            ),

            Expr::Binary(op, lhs, rhs) => {
                let lhs = self.compile_expr(lhs)?;
                let rhs = self.compile_expr(rhs)?;

                Some(match op {
                    Operator::Add => self.builder.build_int_add(lhs, rhs, "addtmp"),
                    Operator::Sub => self.builder.build_int_sub(lhs, rhs, "subtmp"),
                    Operator::Mul => self.builder.build_int_mul(lhs, rhs, "multmp"),
                    Operator::Div => self.builder.build_int_signed_div(lhs, rhs, "divtmp"),
                })
            }

            Expr::Call(name, args) => {
                let fun = self.get_function(&name.name)?;

                let mut compiled_args = Vec::with_capacity(args.len());

                for arg in args {
                    compiled_args.push(self.compile_expr(arg)?);
                }

                let argsv: Vec<BasicMetadataValueEnum> = compiled_args
                    .iter()
                    .by_ref()
                    .map(|&val| val.into())
                    .collect();

                Some(
                    self.builder
                        .build_call(fun, &argsv, "tmp")
                        .try_as_basic_value()
                        .left()?
                        .into_int_value(),
                )
            }
        }
    }

    fn compile_stmt(&mut self, stmt: &Stmt) -> Option<()> {
        match stmt {
            Stmt::Expr(expr) => {
                self.compile_expr(expr)?;
            }
            Stmt::Let(name, expr) => todo!(),
            Stmt::Return(expr) => {
                let expr = self.compile_expr(expr)?;
                self.builder.build_return(Some(&expr));
            }
            Stmt::If(cond, then_b, else_b) => {
                let parent = self.fn_value();
                let zero = self.context.i64_type().const_int(0, true);

                let cond = self.compile_expr(cond)?;
                let cond = self
                    .builder
                    .build_int_compare(IntPredicate::NE, cond, zero, "ifcond");

                let then_bb = self.context.append_basic_block(parent, "then");
                let else_bb = self.context.append_basic_block(parent, "else");
                let cont_bb = self.context.append_basic_block(parent, "ifcont");

                self.builder
                    .build_conditional_branch(cond, then_bb, else_bb);

                //build then
                self.builder.position_at_end(then_bb);
                for stmt in then_b {
                    self.compile_stmt(stmt)?;
                }
                self.builder.build_unconditional_branch(cont_bb);

                // let then_bb = self.builder.get_insert_block().unwrap();

                //build else
                self.builder.position_at_end(else_bb);
                for stmt in else_b {
                    self.compile_stmt(stmt)?;
                }
                self.builder.build_unconditional_branch(cont_bb);

                // let else_bb = self.builder.get_insert_block().unwrap();

                // emit merge block
                self.builder.position_at_end(cont_bb);
                // TODO: make properly phi functions
            }
            Stmt::For(pre_stmt, cond, step, block) => {
                let parent = self.fn_value();

                self.compile_stmt(pre_stmt)?;

                let loop_bb = self.context.append_basic_block(parent, "loop");

                self.builder.build_unconditional_branch(loop_bb);
                self.builder.position_at_end(loop_bb);

                let after_bb = self.context.append_basic_block(parent, "afterloop");

                let cond = self.compile_expr(cond)?;
                let zero = self.context.i64_type().const_int(0, true);

                let cond = self
                    .builder
                    .build_int_compare(IntPredicate::NE, cond, zero, "loopcond");

                self.builder
                    .build_conditional_branch(cond, loop_bb, after_bb);

                for stmt in block {
                    self.compile_stmt(stmt)?;
                }

                self.compile_expr(step)?;

                self.builder.build_unconditional_branch(loop_bb);

                self.builder.position_at_end(after_bb);
            }
            Stmt::Assign(_, _) => todo!(),
        }
        Some(())
    }

    pub fn make_prototype(&mut self, function: &Function) -> Option<FunctionValue<'ctx>> {
        let ret_type = self.context.i64_type();

        let args_types: Vec<BasicMetadataTypeEnum> = std::iter::repeat(ret_type)
            .take(function.args.len())
            .map(|x| x.into())
            .collect();

        let fn_type = self.context.i64_type().fn_type(&args_types, false);
        let fn_val = self.module.add_function(&function.name.name, fn_type, None);

        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_int_value()
                .set_name(function.args[i].name.as_str());
        }

        Some(fn_val)
    }

    pub fn compile_function(&mut self, function: &Function) -> Option<FunctionValue<'ctx>> {
        let func = self.make_prototype(function)?;

        let entry = self.context.append_basic_block(func, "entry");
        self.builder.position_at_end(entry);

        self.fn_value_opt = Some(func);
        self.variables.reserve(function.args.len());

        for (i, arg) in func.get_param_iter().enumerate() {
            let arg_name = function.args[i].name.as_str();
            let alloca = self.create_entry_block_alloca(arg_name);

            self.builder.build_store(alloca, arg);

            self.variables.insert(function.args[i].name.clone(), alloca);
        }

        for stmt in &function.body {
            self.compile_stmt(stmt)?;
        }

        if func.verify(true) {
            self.fpm.run_on(&func);
            Some(func)
        } else {
            println!("not verified");
            unsafe {
                func.delete();
            }
            None
        }
    }

    pub fn compile(
        context: &'ctx Context,
        builder: &'a Builder<'ctx>,
        pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        module: &'a Module<'ctx>,
        program: &Program,
    ) -> Option<FunctionValue<'ctx>> {
        let mut compiler = Self {
            context,
            builder,
            fpm: pass_manager,
            module,
            fn_value_opt: None,
            variables: HashMap::new(),
        };

        compiler.compile_function(&program.functions[0])
    }
}
