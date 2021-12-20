use lang::lexer::tokenize;

mod lang;


fn main() {
  let src = "123";
  let toks = tokenize(src);
}
