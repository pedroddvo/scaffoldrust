use std::{iter::{Peekable, Enumerate}, str::Chars};
use super::error::*;
use itertools::Itertools;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind {
  Eof,
  Newline,

  Number,
  Symbol,
  Type,
  Operator,
  
  KeywordDef,
  KeywordEnd,
  
  Semicolon, // ';'
  Comma,     // ','
  Namespace, // '::'
  Arrow,     // '->'
  
  LParen, // '('
  RParen, // ')'
}

#[derive(Debug, Clone, Copy)]
pub struct Token {
  pub kind: TokenKind,
  pub span: Span,
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
  
  fn ws_not_newline(&mut self) -> bool {
    let p = self.peek_eof().1;
    p != '\n' && p.is_whitespace()
  }
  
  fn token(&mut self) -> IResult<Token> {
    while self.ws_not_newline() { self.next_eof(); }
    let ceof = self.chars.peek();
    if ceof.is_none() {
      return Ok(Token { kind: TokenKind::Eof, span: span_single(self.src.len()) });
    }
    
    let (start, c) = *ceof.unwrap();
    
    if c == '\n' {
      while self.peek_eof().1 == '\n' {
        self.next_eof();
      }
      
      return Ok(Token { kind: TokenKind::Newline, span: span_single(self.src.len()) });
    }
    
    // IF CHARACTER IS DIGIT
    if c.is_digit(10) {
      while self.peek_eof().1.is_digit(10) { self.next_eof(); }
      let after = self.peek_eof().1;
 
      // TODO: more than only base 10
      if after.is_alphabetic() {
        while !self.ws_or_eof() { self.next_eof(); }
        
        let end = self.position();
        return Err(lang_error("Unknown digit type", span(start, end)))
      }
      
      // If the number is a floating point...
      if self.peek_eof().1 == '.' {
        self.chars.next();
        if !self.peek_eof().1.is_digit(10) {
          return Err(lang_error("Expected a digit", span_single(self.position())))
        }
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

      let s = &self.src[start..end];
      if s.chars().next().unwrap().is_uppercase() {
        return Ok(Token { kind: TokenKind::Type, span: span(start, end) })
      }
      let kind = match s {
        "def" => TokenKind::KeywordDef,
        "end" => TokenKind::KeywordEnd,
        _ => TokenKind::Symbol,
      };
      
      return Ok(Token { kind, span: span(start, end) });
    } else {
      let kind = match self.peek_eof().1 {
        ';' => TokenKind::Semicolon,
        ',' => TokenKind::Comma,
        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        ':' => {
          match self.peek_eof().1 {
            ':' => { self.next_eof(); TokenKind::Namespace }
            _ => TokenKind::Operator,
          }
        }
        '-' => {
          match self.peek_eof().1 {
            '>' => { self.next_eof(); TokenKind::Arrow }
            _ => TokenKind::Operator,
          }
        }
        _ => TokenKind::Operator
      };

      if kind != TokenKind::Operator {
        self.next_eof();
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