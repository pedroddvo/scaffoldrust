use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;

use cranelift::codegen::binemit::{NullTrapSink, NullStackMapSink};
use cranelift::codegen::ir::Function;
use cranelift::codegen::{Context, verify_function, binemit};
use cranelift::frontend::{FunctionBuilderContext, FunctionBuilder, Variable};
use cranelift::prelude::{Signature, AbiParam, Value, settings, ExternalName, EntityRef};
use cranelift::prelude::isa::{CallConv, TargetIsa};
use cranelift::prelude::isa;
use cranelift::prelude::types::*;
use cranelift_module::{Module, DataContext, Linkage};
use cranelift::prelude::InstBuilder;
use cranelift_object::{ObjectModule, ObjectBuilder};

use crate::lang::parse::{Expr, ExprKind, self};
use crate::lang::{self, error::{LangError, lang_error_fatal, Span}};

pub struct CodeGen {
  builder_context: FunctionBuilderContext,
  data_ctx: DataContext,
  ctx: Context,
  module: ObjectModule,
}

impl CodeGen {
  fn sig() -> Signature {
    Signature::new(CallConv::SystemV)
  }

  pub fn new() -> Self {
    let flags = settings::Flags::new(settings::builder());
    let isa_builder = isa::lookup_by_name("x86_64-elf").unwrap();
    let isa = isa_builder.finish(flags.clone());
    let obj_builder = ObjectBuilder::new(isa, "output.o", cranelift_module::default_libcall_names()).unwrap();
    Self {
      builder_context: FunctionBuilderContext::new(),
      data_ctx: DataContext::new(),
      ctx: Context::new(),
      module: ObjectModule::new(obj_builder),
    }
  }
  
  fn translate(&mut self, exprs: Vec<Expr>) -> Result<String, LangError> {
    self.ctx.func.signature.returns.push(AbiParam::new(I64));
    
    let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

    let entry_block = builder.create_block();
    builder.append_block_params_for_function_params(entry_block);
    builder.switch_to_block(entry_block);
    builder.seal_block(entry_block);
    
    let mut buf = HashMap::new();
    declare_variables(&mut builder, exprs.as_slice(), &mut 0, &mut buf);
    let mut translator = FunctionTranslator {
      builder,
      variables: buf,
      module: &mut self.module,
      ir: String::new(),
    };
    

    
    let mut ret: Option<Value> = None;
    for expr in exprs {
      ret = Some(translator.translate_expr(expr)?);
    }
    
    let r = ret.unwrap_or(translator.builder.ins().iconst(I64, 0));
    translator.builder.ins()
      .return_(&[r]);
    
    translator.builder.finalize();
    Ok(translator.ir)
  }
  
  pub fn compile(mut self, src: &str) -> Result<(), LangError> {
    let parsed = lang::parse::parse(src)?;
    let ir = self.translate(parsed)?;
    let mut file = File::create(Path::new("gen.clir")).unwrap();
    write!(&mut file, "{}", ir).unwrap();
    write!(&mut file, "{}", self.ctx.func.display().to_string()).unwrap();
    
    let id = self.module
      .declare_function("main", Linkage::Export, &self.ctx.func.signature)
      .unwrap();
    
    self.module
      .define_function(id, &mut self.ctx, &mut NullTrapSink{}, &mut NullStackMapSink{})
      .unwrap();
      
    self.module.clear_context(&mut self.ctx);
    
    let product = self.module.finish();
    let mut file = File::create(Path::new("output.o")).unwrap();
    let emitted = product.emit().unwrap();
    file.write(emitted.as_slice()).unwrap();

    Ok(())
  }
}

struct FunctionTranslator<'a> {
  variables: HashMap<Vec<String>, Variable>,
  builder: FunctionBuilder<'a>,
  module: &'a mut ObjectModule,
  ir: String,
}

impl<'a> FunctionTranslator<'a> {
  fn translate_expr(&mut self, expr: Expr) -> Result<Value, LangError> {
    match expr.kind {
      ExprKind::Number(num) => {
        let imm: i64 = num.parse().unwrap();
        Ok(self.builder.ins().iconst(I64, i64::from(imm)))
      },
      ExprKind::BinaryInfix(lhs, op, rhs) => {
        let llhs = self.translate_expr(*lhs)?;
        let lrhs = self.translate_expr(*rhs)?;
        match op.as_str() {
          "+" => {
            Ok(self.builder.ins().iadd(llhs, lrhs))
          }
          _ => Err(lang_error_fatal("Unknown infix operator", expr.span)),
        }
      },
      ExprKind::FuncDef(namespaced, params, ret_type, stmts) => {
        let mut ctx = Context::new();
        for (_, ty) in params.clone() {
          ctx.func.signature.params.push(AbiParam::new(sfdtype_to_code_type(ty).unwrap()));
        }
        
        if let Some(r) = ret_type {
          ctx.func.signature.returns.push(AbiParam::new(sfdtype_to_code_type(r).unwrap()));
        } else {
          ctx.func.signature.returns.push(AbiParam::new(I64));
        }
        
        let mut bctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut bctx);
        let entry = builder.create_block();
        builder.append_block_params_for_function_params(entry);
        builder.switch_to_block(entry);
        builder.seal_block(entry);
        
        let mut buf = self.variables.clone();

        let mut i = 0;
        for (name, ty) in params {
          let var = Variable::new(i);
          buf.insert(vec![name], var);
          builder.declare_var(var, sfdtype_to_code_type(ty).unwrap());
          i += 1;
        }

        declare_variables(&mut builder, &stmts, &mut i, &mut buf);
        
        
        let mut trans = FunctionTranslator {
          variables: buf,
          builder,
          module: self.module,
          ir: String::new(),
        };
        
        let mut ret: Option<Value> = None;
        for expr in stmts {
          ret = Some(trans.translate_expr(expr)?);
        }
        
        let r = ret.unwrap_or(trans.builder.ins().iconst(I64, 0));
        trans.builder.ins()
          .return_(&[r]);
      
        trans.builder.finalize();
        
        let id = self.module
          .declare_function(&namespaced[0], Linkage::Local, &ctx.func.signature)
          .unwrap();
        
        self.module
          .define_function(id, &mut ctx, &mut NullTrapSink{}, &mut NullStackMapSink{})
          .unwrap();
        
        self.ir += &(ctx.func.display().to_string() + "\n");
        
        self.module.clear_context(&mut ctx);
        
        // let vfs = self.variables.get(&namespaced).unwrap();
        // self.builder.def_var(*vfs, Value::from_u32(id.as_u32()));
        
        Ok(self.builder.ins().iconst(I64, 0))
      }
      ExprKind::FuncCall(namespaced, args) => {
        if !self.variables.contains_key(&namespaced) {
          return Err(lang_error_fatal("Undefined function", expr.span));
        }
        
        let fvar = self.variables.get(&namespaced).unwrap();
        unimplemented!()
      },
      ExprKind::Symbol(sym) => {
        let var = self.variables.get(&sym)
          .ok_or(lang_error_fatal("Undefined variable", expr.span))?;
        Ok(self.builder.use_var(*var))
      }
      _ => unimplemented!()
    }
  }
}

fn declare_variables(builder: &mut FunctionBuilder, exprs: &[Expr], index: &mut usize, vars: &mut HashMap<Vec<String>, Variable>) {
  for expr in exprs {
    match expr.clone().kind {
      ExprKind::FuncDef(namespaced, params, ret_type, _) => {
        if !vars.contains_key(&namespaced) {
          let var = Variable::new(*index);
          vars.insert(namespaced, var);
          builder.declare_var(var, I64);
          *index += 1;
        }
      }
      _ => {}
    }
  }
}

fn sfdtype_to_code_type(ty: String) -> Option<Type> {
  match ty.as_str() {
    "Int32" => Some(I32),
    "Int64" => Some(I64),
    _ => unimplemented!("Add custom types!")
  }
}
