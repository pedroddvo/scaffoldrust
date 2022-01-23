use std::{collections::HashMap, convert::TryInto, fs::File, path::Path};

use inkwell::{
  context::Context, 
  passes::PassManager,
  values::{FunctionValue, PointerValue, AnyValue, AnyValueEnum, IntValue, BasicValue}, 
  module::Module,
  execution_engine::FunctionLookupError,
  builder::Builder, types::{AnyTypeEnum, IntType}, targets::{TargetMachine, Target, RelocMode, CodeModel, FileType}, OptimizationLevel
};
use std::str::FromStr;

use crate::lang::{error::{LangError, lang_error_fatal, span, span_single}, parse::{Parser, Expr, ExprKind}, self};

pub struct Compiler<'a, 'ctx> {
  pub context: &'ctx Context,
  pub builder: &'a Builder<'ctx>,
  pub fpm: &'a PassManager<FunctionValue<'ctx>>,
  pub module: &'a Module<'ctx>,

  /*variables: HashMap<String, PointerValue<'ctx>>,*/
  /*fn_value_*/
}

use AnyValueEnum::*;
impl<'a, 'ctx> Compiler<'a, 'ctx> {
  fn compile_expr(&mut self, expr: &Expr) -> Result<AnyValueEnum<'ctx>, LangError> {
    match &expr.kind {
      ExprKind::Number(num) => {
        Ok(IntValue(self.context.i64_type().const_int(num.parse().unwrap(), false)))
      }
      
      _ => unimplemented!()
    }
  }
  
  fn compile_main(&mut self, stmt: &Expr) -> Result<FunctionValue<'ctx>, LangError> {
    let main_type = self.module.add_function(
      "main",
      self.context.i32_type().fn_type(&[], false),
      None
    );
    let entry = self.context.append_basic_block(main_type, "entry");
    
    self.builder.position_at_end(entry);
    let body = self.compile_expr(stmt)?;
    
    self.builder.build_return(None);
    //match body {
      //IntValue(n) => {
        //let casted = n.const_cast(self.context.i32_type(), true);
        //self.builder.build_return(Some(&casted));
      //},
      //_ => unimplemented!()
    //}

    if main_type.verify(true) {
      self.fpm.run_on(&main_type);
      
      Ok(main_type)
    } else {
      unsafe {
        main_type.delete();
      }
      
      panic!("Invalid generated function!")
    }
    

  }

  pub fn compile(
    context: &'ctx Context,
    builder: &'a Builder<'ctx>,
    pass_manager: &'a PassManager<FunctionValue<'ctx>>,
    module: &'a Module<'ctx>,
    main: &Expr
  ) -> Result<FunctionValue<'ctx>, LangError> {  
    let mut compiler = Compiler {
      context,
      builder,
      fpm: pass_manager,
      module,
    };

    compiler.compile_main(main)
  }
}

pub fn code_gen(input: &str) -> Result<(), LangError> {
  let context = Context::create();
  let module = context.create_module("execute");
  let builder = context.create_builder();

  let fpm = PassManager::create(&module);

  fpm.add_instruction_combining_pass();
  fpm.add_reassociate_pass();
  fpm.add_gvn_pass();
  fpm.add_cfg_simplification_pass();
  fpm.add_basic_alias_analysis_pass();
  fpm.add_promote_memory_to_register_pass();
  fpm.add_instruction_combining_pass();
  fpm.add_reassociate_pass();

  {
    let parsed = lang::parse::parse(input)?;
    let expr = &parsed[0];

    let module = context.create_module("tmp");
    let f = Compiler::compile(&context, &builder, &fpm, &module, expr)?;
    
    f.print_to_stderr();
    
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).unwrap();
    let target_machine = target.create_target_machine(&triple, "generic", "", OptimizationLevel::None, RelocMode::Default, CodeModel::Default).unwrap();

    
    module.set_data_layout(&target_machine.get_target_data().get_data_layout());
    module.set_triple(&triple);
    
    target_machine.add_analysis_passes(&fpm);
    target_machine.write_to_file(&module, FileType::Object, Path::new("output.o")).unwrap();
    
    fpm.initialize();
  }
  
  Ok(())

}