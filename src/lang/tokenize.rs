use std::{iter::{Peekable, Enumerate}, str::Chars};
use super::error::{LangError, Span, lang_error, lang_error_fatal, span, span_single, LangErrorKind};
use itertools::Itertools;

#[derive(Debug, PartialEq, Eq)]
pub enum TokenKind {
  Eof,
  Number,
  Symbol,
  Operator,
  
  LParen,
  RParen,
}

#[derive(Debug)]
pub struct Token {
  kind: TokenKind,
  span: Span,
}


struct Tokenizer<'a> {
  src: String,
  chars: Peekable<Enumerate<Chars<'a>>>
}

impl<'a> Tokenizer<'a> {
  fn peek_eof(&mut self) -> (usize, char) {
    self.chars.peek().unwrap_or(&(self.src.len(), '\0')).clone()
  }
  
  fn next_eof(&mut self) -> (usize, char) {
    self.chars.next().unwrap_or((self.src.len(), '\0'))
  }
  
  fn position(&mut self) -> usize {
    self.peek_eof().0
  }
  
  fn ws_or_eof(&mut self) -> bool {
    self.chars.peek().map(|(_, c)| c.is_whitespace()).unwrap_or(true)
  }
  
  fn token(&mut self) -> Result<Token, LangError> {
    while self.peek_eof().1.is_whitespace() { self.next_eof(); }
    let ceof = self.chars.peek();
    if ceof.is_none() {
      return Ok(Token { kind: TokenKind::Eof, span: span_single(self.src.len()) });
    }
    
    let (start, c) = *ceof.unwrap();
    
    if c.is_digit(10) {
      while self.peek_eof().1.is_digit(10) { self.next_eof(); }
      let after = self.peek_eof().1;
      if !after.is_whitespace() && after != '\0' {
        while !self.ws_or_eof() { self.next_eof(); }
        
        let end = self.position();
        return Err(lang_error("Unknown digit type", span(start, end)))
      }
      
      // If the number is a floating point...
      if self.peek_eof().1 == '.' {
        self.chars.next();
        while self.peek_eof().1.is_digit(10) { self.next_eof(); }

        let end = self.position();
        return Ok(Token { kind: TokenKind::Number, span: span(start, end) })
      }

      let end = self.position();
      return Ok(Token { kind: TokenKind::Number, span: span(start, end)});
    } else if c.is_alphabetic() {
      self.next_eof();
      while self.peek_eof().1.is_alphanumeric() { self.next_eof(); }
      
      // e.g isdigit?
      match self.peek_eof().1 {
        '?' => { self.next_eof(); },
        _ => {}
      }

      let end = self.position();
      return Ok(Token { kind: TokenKind::Symbol, span: span(start, end)});
    } else {
      let kind = match self.peek_eof().1 {
        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        _ => TokenKind::Operator
      };

      if kind != TokenKind::Operator {
        return Ok(Token { kind, span: span(start, self.position())})
      }
      
      // constructable operators
      loop {
        match self.peek_eof().1 {
          '+' | '-' | '*' | '/' => { self.next_eof(); }
          _ => break
        }
      }
      
      let end = self.position();
      if end == start { 
        return Err(lang_error("Bad character(s)", span(start, self.position())))
      }
      
      Ok(Token { kind, span: span(start, end) })
    }
    
  }
}

pub fn tokenize(src: &str) -> Result<Vec<Token>, Vec<LangError>> {
  let mut t = Tokenizer { src: src.to_string(), chars: src.chars().enumerate().peekable() };
  let mut toks = vec![];
  let mut errs = vec![];
  
  'inf: loop {
    let errtok = t.token();
    
    match errtok {
      Err(err) => {
        if let LangErrorKind::Fatal = err.kind {
          errs.push(err);
          break 'inf;
        }
        errs.push(err);
      }
      Ok(tok) => {
        if let TokenKind::Eof = tok.kind {
          toks.push(tok);
          break 'inf;
        }
        toks.push(tok);
      }
    }
  }
  
  if !errs.is_empty() {
    return Err(errs);
  }
  
  Ok(toks)
}