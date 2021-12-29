use lang::error::{report_error, LangErrorKind};

mod lang;
mod codegen;


fn main() {
  let src = std::fs::read_to_string("example.sfd").unwrap();
  
  let n = lang::parse::parse(&src);
  match n {
    Ok(e) => println!("{:#?}", e),
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
