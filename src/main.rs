use core::mem;
use codegen::CodeGen;
use lang::error::{report_error, LangErrorKind, LangError};

mod lang;
mod codegen;


fn main() {
  let src = std::fs::read_to_string("example.sfd").unwrap();
  
  let ran = CodeGen::new().compile(&src);

  match ran {
    Ok(e) => {
      
    },
    Err(err) => {
      match err.clone().kind {
        LangErrorKind::Contextual(errs) => {
          println!("{}", report_error(&src, err));
          for e in errs {
            println!("{}", report_error(&src, e));
          }
        }
        _ => {
          println!("{}", report_error(&src, err))
        }
      }
    }
  }
}
