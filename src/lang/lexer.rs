use std::convert::TryInto;
use std::iter::Peekable;
use super::token::{Token, TokenKind};
use super::error::{LangError, LangTrace};



pub struct Lexer<'a> {
  trace: LangTrace,

  src: &'a [u8],
  src_offset: usize,
}

impl<'a> Lexer<'a> {
  fn eof(&self) -> bool {
    return self.src_offset >= self.src.len();
  }

  fn next(&mut self) -> u8 {
    if self.eof() {
      return 0;
    }

    self.src_offset += 1;
    return self.src[self.src_offset-1];
  }
  
  fn peek(&self) -> u8 {
    if self.eof() {
      return 0;
    }
    
    return self.src[self.src_offset];
  }
  
  fn whitespace(&mut self) {
    while self.peek().is_ascii_whitespace() {
      self.next();
    }
  }
  
  fn gen_token(&self, start: usize, kind: TokenKind) -> Token {
    // this will only overflow if there's greater than 2,147,483,647 tokens
    Token { kind, start, offset: self.src_offset.try_into().unwrap() }
  }
  
  fn is_unreserved_op(&self, b: u8) -> bool {
    match b {
      b'+' | b'-' | b'*' | b'/' => true,
      _ => false,
    }
  }

  fn token(&mut self) -> Token {
    self.whitespace();

    let c = self.peek();
    let start = self.src_offset;

    self.whitespace();

    if c.is_ascii_digit() {
      while self.peek().is_ascii_digit() {
        self.next();
      }
      self.gen_token(start, TokenKind::Number)
    } else {
      let reserved_op = match c {
        b'(' => Some(TokenKind::LParen),
        b')' => Some(TokenKind::RParen),
        b'{' => Some(TokenKind::LCurly),
        b'}' => Some(TokenKind::RCurly),
        _ => None
      };

      if let Some(kind) = reserved_op {
        return self.gen_token(start, kind);
      }


      while !self.eof() && !self.peek().is_ascii_whitespace() {
        self.next();
      }
      
      self.trace.error(start, self.src_offset.try_into().unwrap(), format!("Bad character(s)!"));
      self.gen_token(start, TokenKind::Error)
    }
  }
}

pub fn tokenize(src: &str) -> Vec<Token> {
  let mut lexer = Lexer { src: src.as_bytes(), src_offset: 0, trace: LangTrace::new() };
  let mut toks = Vec::<Token>::new();

  loop {
    toks.push(lexer.token());
    if lexer.eof() {
      break;
    }
  }

  toks
}