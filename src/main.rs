extern crate nom;
mod lang;


fn main() {
  let src = "1 + 2 * 3";
  match lang::parse::parse(src) {
    Ok(o) => {
      println!("Success:\n{}\n{:?}", o.0, o.1);
    }
    Err(nom::Err::Error(e)) => {
      println!("{}", lang::error::report_error(src, e));
    }
    _ => {}
  }
}
