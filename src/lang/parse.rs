use nom::{
  bytes::complete::{tag, take_while, take_while1},
  character::complete::{char, anychar, space0, space1, newline},
  error::{context, Error, ErrorKind, VerboseError, VerboseErrorKind, ParseError},
  multi::{many0},
  combinator::{peek},
  sequence::{preceded},
  branch::{alt},
  Parser,
};

#[derive(Debug)]
pub enum Expr {
  Number(f64),
  Symbol(String),
  
  Binary(Box<Expr>, String, Box<Expr>),
}

type IResult<'a, I, O, E = &'a str> = nom::IResult<I, O, VerboseError<E>>;

fn gen_verbose<'a>(i: &'a str, ctx: &'static str) -> nom::Err<VerboseError<&'a str>> {
  nom::Err::Error(VerboseError { errors: vec![(i, VerboseErrorKind::Context(ctx))]})
}

// skips a single space, tab, newline
fn nlspace0(i: &str) -> IResult<&str, &str> {
  take_while(|c: char| c.is_whitespace())(i)
}

fn ws<'a, F: 'a, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where F: Fn(&'a str) -> IResult<&'a str, O> {
  preceded(space0, inner)
}

fn skip_ws(i: &str) -> &str {
  space0::<&str, VerboseError<&str>>(i).unwrap().0
}

fn peek_next(i: &str) -> IResult<&str, char> {
  anychar::<&str, VerboseError<&str>>(i)
}
  
fn parse_dec_digits(i: &str) -> IResult<&str, &str> {
  let (i, d) = take_while1(|c: char| c.is_digit(10))(i)?;
  Ok((i, d))
}

fn parse_dec_float(i: &str) -> IResult<&str, String> {
  let (i, head) = parse_dec_digits(i)?;
  let td = peek_next(i);
  if let Ok((i2, c)) = td {
    if c == '.' {  
      let (i2, tail) = context("Decimal digit 0 - 9", parse_dec_digits)(i2)?;
      return Ok((i2, format!("{}.{}", head, tail)));
    }
  }
  
  Ok((i, String::from(head)))
}
  
fn parse_symbol(i: &str) -> IResult<&str, String> {
  let (i, head) = 
    context("Alphabetic character", take_while1(|c: char| c.is_alphabetic()))(i)?;
  let (i, tail) = 
    take_while(|c: char| c.is_alphanumeric() || c == '?')(i)?;
  
  Ok((i, format!("{}{}", head, tail)))
}

fn parse_atomic(i: &str) -> IResult<&str, Expr> {
  let (_, c) = anychar(i)?;
  
  if c.is_alphabetic() {
    parse_symbol(i).map(|r| (r.0, Expr::Symbol(r.1)))
  } else if c.is_digit(10) {
    let (i, f) = parse_dec_float(i)?;
    let n = f.parse::<f64>().map_err(|_| gen_verbose(i, "Valid float"))?;
    Ok((i, Expr::Number(n)))
  } else {
    Err(gen_verbose(i, "Digit or Alphabetic character"))
  }
}

fn infix_bp(op: char) -> Option<(u8, u8)> {
  match op {
    '+' | '-' => Some((10, 11)),
    '*' | '/' => Some((11, 12)),
    _ => None,
  }
}

fn parse_binary(i: &str, min_bp: u8) -> IResult<&str, Expr> {
  let (mut i, mut lhs) = parse_atomic(i)?;
  
  loop {
    let i2 = nlspace0(i)?.0;
    let (i2, op) = take_while(|c: char| infix_bp(c).is_some())(i2)?;
    if op == "" { break; }
    
    if let Some((l_bp, r_bp)) = infix_bp(op.chars().next().unwrap()) {
      if l_bp < min_bp {
        break;
      }
      i = nlspace0(i2)?.0;
      
      let rhs = parse_binary(i, r_bp)?;
      i = rhs.0;
      lhs = Expr::Binary(Box::new(lhs), op.to_string(), Box::new(rhs.1));
    } else {
      return Err(gen_verbose(op, "Valid infix operator"))
    }

  }
  Ok((i, lhs))
}

fn expr(i: &str) -> IResult<&str, Expr> {
  parse_binary(i, 0)
}

pub fn parse(src: &str) -> IResult<&str, Expr> {
  expr(src)
}