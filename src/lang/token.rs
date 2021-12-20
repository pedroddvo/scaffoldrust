#[derive(Debug, Clone)]
pub enum TokenKind {
  Error, Eof, Sof,
  
  LParen,
  RParen,
  LCurly,
  RCurly,

  Operator,
  Number,
}

#[derive(Debug, Clone)]
pub struct Token {
  pub kind: TokenKind,
  pub start: usize,
  pub offset: isize,
}