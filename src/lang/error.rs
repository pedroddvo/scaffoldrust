use std::io::BufRead;

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

#[derive(Debug, Clone)]
pub enum LangErrorKind {
  Many(Vec<LangError>),
  Contextual(Vec<LangError>),
  Fatal,
}

#[derive(Debug, Clone)]
pub struct LangError {
  pub kind: LangErrorKind,
  pub msg: String,
  pub span: Span
}

pub type IResult<I> = Result<I, LangError>;

pub fn lang_error(msg: &str, span: Span) -> LangError {
  LangError { msg: msg.to_string(), span, kind: LangErrorKind::Contextual(vec![]) }
}

pub fn lang_errors(span: Span, errors: Vec<LangError>) -> LangError {
  LangError { msg: String::new(), span, kind: LangErrorKind::Many(errors) }
}

pub fn lang_error_fatal(msg: &str, span: Span) -> LangError {
  LangError { msg: msg.to_string(), span, kind: LangErrorKind::Fatal }
}

pub fn report_error(src: &str, err: LangError) -> String {

  use std::fmt::Write;

  let mut buf = String::new();
  
  if let LangErrorKind::Many(e) = err.kind.clone() {
    for me in e {
      write!(&mut buf, "{}\n", report_error(src, me)).unwrap();
    }
    return buf;
  }

  let prefix = &src.as_bytes()[..err.span.start];
  let line_number = prefix.iter().filter(|&&c| c == b'\n').count() + 1;
    
  let line_begin = prefix.iter().rev()
    .position(|&b| b == b'\n')
    .map(|p| err.span.start - p)
    .unwrap_or(0);
    
    
  let line = src[line_begin..].lines().next().unwrap_or(&src[line_begin..]).trim_end();
    
  let column = err.span.start - line_begin;


  write!(&mut buf, "{}:{}: error: {}\n", 
    line_number,
    column+1,
    err.msg,
  ).unwrap();
  write!(&mut buf, "\t{}\n", line).unwrap();
  if err.span.start == err.span.end {
    write!(&mut buf, "\t{}^", " ".repeat(column)).unwrap();
  } else {
    write!(&mut buf, "\t{}{}", " ".repeat(column), "~".repeat(err.span.end - err.span.start)).unwrap();
  }

  buf
}