extern crate nom;
mod lang;

fn main() {
  let x = lang::parse::Parser::parse("Hello");
  println!("{:?}", x);
}
