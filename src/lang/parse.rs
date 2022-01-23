use std::{iter::Peekable, slice::Iter};

use super::tokenize::{Token, TokenKind, tokenize};
use super::error::*;

#[derive(Debug, Clone)]
pub enum ExprKind {
  Number(String),
  Symbol(Vec<String>),
   
  BinaryInfix(Box<Expr>, String, Box<Expr>),

  FuncDef(Vec<String>, Vec<(String, String)>, Option<String>, Vec<Expr>), // namespaced name, typed parameters, (return type), stmts
  FuncCall(Vec<String>, Vec<Expr>), // namespaced name, args
}

#[derive(Debug, Clone)]
pub struct Expr {
  pub kind: ExprKind,
  pub span: Span,
}

pub struct Parser<'a> {
  src: String,
  tokens: Peekable<Iter<'a, Token>>
}

fn infix_bp(op: char) -> Option<(u8, u8)> {
  match op {
    '+' | '-' => Some((10, 11)),
    '*' | '/' => Some((12, 13)),
    _ => None,
  }
}

impl<'a> Parser<'a> {
  fn peek_no_eof(&mut self) -> IResult<Token> {
    match self.tokens.peek() {
      None => Err(lang_error("Unexpected end of file", span_single(self.src.len()))),
      Some(t) => { 
        if t.kind == TokenKind::Eof {
          return Err(lang_error("Unexpected end of file", span_single(self.src.len())));
        }
        Ok(**t)
      },
    } 
  }
  
  fn next_if(&mut self, kind: TokenKind) -> IResult<Token> {
    let t = self.peek_no_eof()?;
    if t.kind == kind {
      return self.next_no_eof();
    }
    
    Ok(t)
  }
  
  fn expect_no_next(&mut self, kind: TokenKind) -> IResult<Token> {
    let t = self.peek_no_eof()?;
    if t.kind == kind {
      return Ok(t);
    }

    Err(lang_error(&format!("Expected {:?} got {:?}!", kind, self.span_str(t.span)), t.span))
  }

  fn expect_next(&mut self, kind: TokenKind) -> IResult<Token> {
    let t = self.expect_no_next(kind)?;
    self.next_no_eof()?;
    Ok(t)
  }
  
  fn expect_or(&mut self, kinds: Vec<TokenKind>) -> IResult<Token> {
    let span = self.peek_no_eof()?.span;
    for kind in kinds.clone() {
      return match self.expect_no_next(kind) {
        Err(e) => continue,
        Ok(o) => Ok(o)
      }
    }
     
    let fmt: Vec<String> = kinds.iter().map(|&t| format!("{:?}", t)).collect();
    let res = fmt.iter().enumerate()
      .map(|(n, s)| { if n == fmt.len()-1 { (*s).clone() } else { format!("{} or ", s) }}).collect::<String>();
    
    Err(lang_error(&format!("Expected {} got {:?}!", res, self.span_str(span)), span))
  }
  
  fn next_no_eof(&mut self) -> IResult<Token> {
    let t = self.peek_no_eof()?;
    self.tokens.next(); 
    Ok(t)
  }
  
  fn peek_no_borrow(&mut self) -> Option<Token> {
    self.tokens.peek().map(|t| **t)
  }
  
  fn span_str(&self, span: Span) -> &str {
    &self.src[span.start..span.end]
  }
   
  fn parse_atomic(&mut self) -> IResult<Expr> {
    while self.peek_no_eof()?.kind == TokenKind::Newline {
      self.next_no_eof()?;
    }
    let tok = self.peek_no_eof()?;
    
    match tok.kind {
      TokenKind::Number => { 
        self.next_no_eof()?;
        Ok(Expr { 
          kind: ExprKind::Number(self.span_str(tok.span).to_string()), 
          span: tok.span 
        })
      },
      TokenKind::Symbol => {
        let namespaced = self.parse_namespace_name()?;
        if self.tokens.peek().is_some() {
          if self.peek_no_eof()?.kind == TokenKind::LParen {
            return self.parse_funccall(namespaced, tok.span.start);
          }
        }

        Ok(Expr { 
          kind: ExprKind::Symbol(namespaced), 
          span: tok.span 
        })
      },
      
      _ => Err(lang_error_fatal("Unexpected token", tok.span))
    }
  }
 
  fn parse_binary(&mut self, min_bp: u8) -> IResult<Expr> {
    let mut lhs = self.parse_atomic()?;
    let start = lhs.span.start;
    
    loop {
      let opt = self.peek_no_borrow();
      if opt.is_none() { break; }
      let opc = opt.unwrap();
      
      let op = match opc.kind {
        TokenKind::Operator => Ok(self.span_str(opc.span)),
        _ => break
      }?.to_string();

      if let Some((l_bp, r_bp)) = infix_bp(op.chars().next().unwrap()) {
        if l_bp < min_bp {
          break;
        }

        self.tokens.next();
        let rhs = self.parse_binary(r_bp)?;
        let end = rhs.span.end;
        
        lhs = Expr {
          kind: ExprKind::BinaryInfix(Box::new(lhs), op, Box::new(rhs)),
          span: span(start, end)
        };
        continue;
      }
      
      break;
    }

    Ok(lhs)
  }
  
  fn parse_expr(&mut self) -> IResult<Expr> {
    self.parse_binary(0)
  }
   
  fn parse_namespace_name(&mut self) -> IResult<Vec<String>> {
    let mut res = vec![];
    loop {
      let s = self.expect_or(vec![TokenKind::Symbol, TokenKind::Type])?;
      self.next_no_eof()?;
      res.push(self.span_str(s.span).to_string());
      let no = self.tokens.peek();
      if let Some(&&n) = no {
        if n.kind == TokenKind::Namespace {
          self.next_no_eof()?; 
          continue
        } else {
          break;
        }
      } else {
        break;
      }

    }
    Ok(res)
  } 
  
  fn parse_parameters(&mut self) -> IResult<Vec<(String, String)>> {
    let mut res = vec![];
    loop {
      if self.peek_no_eof()?.kind != TokenKind::Symbol {
        break;
      }
      let name = self.expect_next(TokenKind::Symbol)?;
      let ntype = self.expect_next(TokenKind::Type)?;
      res.push((self.span_str(name.span).to_string(), self.span_str(ntype.span).to_string()));
      if let TokenKind::Comma = self.peek_no_eof()?.kind {
        self.next_no_eof()?;
      } else {
        break;
      }
    }
    Ok(res)
  }
  
  fn parse_arguments(&mut self) -> IResult<Vec<Expr>> {
    let mut res = vec![];
    loop {
      let expr = self.parse_expr()?;
      res.push(expr);
      if let TokenKind::Comma = self.peek_no_eof()?.kind {
        self.next_no_eof()?;
      } else {
        break;
      }
    }
    Ok(res)
  }

  fn parse_stmts_till(&mut self, kind: TokenKind) -> IResult<Vec<Expr>> {
    let mut res = vec![];
    let mut errs = vec![];
    while self.peek_no_eof()?.kind != kind {
      let stmt = self.parse_stmt();
      match stmt {
        Ok(e) => res.push(e),
        Err(e) => {
          if let LangErrorKind::Fatal = e.kind {
            return Err(e);
          }
          errs.push(e);
        }
      } 

      if errs.is_empty() {
        self.expect_or(vec![TokenKind::Semicolon, TokenKind::Newline])?;
        self.next_no_eof()?;
      }
    }
    
    Ok(res)
  }
  
  fn parse_funccall(&mut self, name: Vec<String>, start: usize) -> IResult<Expr> {
    self.expect_next(TokenKind::LParen)?;
    if self.peek_no_eof()?.kind == TokenKind::RParen {
      let end = self.next_no_eof()?.span.end;
      return Ok(Expr {
        kind: ExprKind::FuncCall(name, vec![]),
        span: span(start, end)
      }) 
    }
    let args = self.parse_arguments()?;
    let end = self.expect_next(TokenKind::RParen)?.span.end;
    return Ok(Expr {
      kind: ExprKind::FuncCall(name, args),
      span: span(start, end)
    }) 
  }

  fn parse_funcdef(&mut self) -> IResult<Expr> {
    let start = self.next_no_eof()?.span.start; // skip 'def'
    let mut params = vec![];
    let mut ntype = None;
    let name = self.parse_namespace_name()?;
    
    let mut t = self.expect_or(vec![TokenKind::LParen, TokenKind::Arrow])?;

    if t.kind == TokenKind::LParen {
      self.next_no_eof()?;
      params = self.parse_parameters()?;
      self.expect_next(TokenKind::RParen);
      t = self.peek_no_eof()?;
    } 
    
    if t.kind == TokenKind::Arrow {
      self.next_no_eof()?;
      let t = self.expect_next(TokenKind::Type)?;
      ntype = Some(self.span_str(t.span).to_string());
    }
    
    self.expect_next(TokenKind::Newline)?;
    
    let exprs = self.parse_stmts_till(TokenKind::KeywordEnd)?;
    let end = self.next_no_eof()?.span.end; // skip 'end'

    Ok(Expr {
      kind: ExprKind::FuncDef(name, params, ntype, exprs),
      span: span(start, end),
    })
  }
  
  fn parse_stmt(&mut self) -> IResult<Expr> {
    let p = self.peek_no_eof()?;
    match p.kind {
      TokenKind::KeywordDef => self.parse_funcdef(),
      _ => self.parse_expr()
    }
  }

  fn parse_exprs(&mut self) -> IResult<Vec<Expr>> {
    let mut exprs = vec![];
    let mut errors = vec![];
    

    loop {
      let expr = self.parse_stmt();
      match expr {
        Ok(e) => exprs.push(e),
        Err(e) => {
          return Err(e)
        },
      }
      
      if self.tokens.peek().unwrap().kind == TokenKind::Eof {

        break;
      }
      if errors.is_empty() {
        self.expect_or(vec![TokenKind::Newline, TokenKind::Semicolon])?;
        self.next_no_eof()?;
      }
      if self.tokens.peek().unwrap().kind == TokenKind::Eof {
        break;
      }
      
      

    }
    
    let start = exprs.first().map(|e| e.span.start).unwrap_or(0);
    let end = exprs.last().map(|e| e.span.end).unwrap_or(0);
    
    if !errors.is_empty() {
      return Err(lang_errors(span(start, end), errors));
    }

    Ok(exprs)
  }
}


pub fn parse(src: &str) -> IResult<Vec<Expr>> {
  let n = tokenize(src);
  match n {
    Ok(toks) => {
      let mut p = Parser { src: src.to_string(), tokens: toks.iter().peekable() };
      
      p.parse_exprs()
    },
    Err(mut errs) => {
      Err(errs.pop().unwrap())
    }
  }
}