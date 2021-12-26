extern crate nom;
extern crate nom_locate;
mod lang;


fn main() {
  let src = std::fs::read_to_string("example.sfd").unwrap();
  match lang::parse::parse(src.as_str()) {
    Ok(o) => {
      println!("Success:\n{:?}\n{:#?}", o.0, o.1);
      
      let mut e = lang::interpreter::Env {};
      println!("Interpreted top: {:?}", e.expr(o.1));
    }
    Err(nom::Err::Error(e)) => {
      println!("{}", lang::error::report_error(src.as_str(), e));
    }
    _ => {}
  }
}
