use lang::error::report_error;

mod lang;


fn main() {
  let src = std::fs::read_to_string("example.sfd").unwrap();
  
  let n = lang::tokenize::tokenize(&src);
  match n {
    Ok(toks) => println!("{:?}", toks),
    Err(errs) => {
      for err in errs {
        println!("{}", report_error(&src, err));
      }
    }
  }
}
