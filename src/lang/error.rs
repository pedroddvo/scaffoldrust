#[derive(Debug, Clone, Copy)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

pub fn span(start: usize, end: usize) -> Span {
  Span { start, end }
}

pub fn span_single(startend: usize) -> Span {
  Span { start: startend, end: startend }
}

#[derive(Debug)]
pub enum LangErrorKind {
  Error,
  Fatal,
}

#[derive(Debug)]
pub struct LangError {
  pub kind: LangErrorKind,
  pub msg: String,
  pub span: Span
}

pub fn lang_error(msg: &str, span: Span) -> LangError {
  LangError { msg: msg.to_string(), span, kind: LangErrorKind::Error }
}

pub fn lang_error_fatal(msg: &str, span: Span) -> LangError {
  LangError { msg: msg.to_string(), span, kind: LangErrorKind::Fatal }
}

pub fn report_error(src: &str, err: LangError) -> String {
  use std::fmt::Write;

  let mut buf = String::new();
  
  let prefix = &src.as_bytes()[..err.span.start];
  let line_number = prefix.iter().filter(|&&c| c == b'\n').count() + 1;
    
  let line_begin = prefix.iter().rev()
    .position(|&b| b == b'\n')
    .map(|p| err.span.start - p)
    .unwrap_or(0);
    
    
  let line_end = src.as_bytes()[line_begin..].iter()
    .position(|&b| b == b'\n')
    .unwrap_or(src.len());
    
  let column = err.span.start - line_begin;


  write!(&mut buf, "{}:{}: error: {}\n", 
    line_number,
    column,
    err.msg,
  ).unwrap();
  write!(&mut buf, "\t{}\n", &src[line_begin..line_end]).unwrap();
  write!(&mut buf, "\t{}^", " ".repeat(column)).unwrap();

  buf
}