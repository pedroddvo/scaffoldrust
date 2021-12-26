use nom::{
  bytes::complete::{tag, take_while, take_while1},
  character::complete::{char, anychar, space0, space1, newline},
  error::{context, Error, ErrorKind, VerboseError, VerboseErrorKind, ParseError},
  multi::{many0, many1},
  combinator::{peek, opt, eof},
  sequence::{preceded, terminated, delimited, separated_pair},
  branch::{alt},
  Parser,
};

use nom_locate::{LocatedSpan, position};

type Span<'a> = LocatedSpan<&'a str>;
type IResult<'a, I, O, E = Span<'a>> = nom::IResult<I, O, VerboseError<E>>;

#[derive(Debug)]
pub struct Expr<'a> {
  pub position: Span<'a>,
  pub kind: ExprKind<'a>,
}

#[derive(Debug)]
pub enum ExprKind<'a> {
  Null,
  Number(f64),
  Symbol(String),
  
  Binary(Box<Expr<'a>>, String, Box<Expr<'a>>),
  Block(Vec<Expr<'a>>),
  FuncDef(Vec<String>, Option<Vec<(String, String)>>, Option<String>, Box<Expr<'a>>), // namespaced name, typed parameters, (return type), expr
}


fn gen_verbose<'a>(i: Span<'a>, ctx: &'static str) -> nom::Err<VerboseError<Span<'a>>> {
  nom::Err::Error(VerboseError { errors: vec![(i, VerboseErrorKind::Context(ctx))]})
}

// skips a single space, tab, newline
fn nlspace0(i: Span) -> IResult<Span, LocatedSpan<&str>> {
  take_while(|c: char| c.is_whitespace())(i)
}

fn nlspace1(i: Span) -> IResult<Span, LocatedSpan<&str>> {
  take_while1(|c: char| c.is_whitespace())(i)
}

fn peek_next(i: Span) -> IResult<Span, char> {
  if eof::<Span, VerboseError<Span>>(i).is_ok() {
    return Ok((i, '\0'))
  }
  anychar(i)
}
  
fn parse_dec_digits(i: Span) -> IResult<Span, LocatedSpan<&str>> {
  let (i, d) = take_while1(|c: char| c.is_digit(10))(i)?;
  Ok((i, d))
}

fn parse_dec_float(i: Span) -> IResult<Span, String> {
  let (i, head) = parse_dec_digits(i)?;
  let td = peek_next(i);
  if let Ok((i2, c)) = td {
    if c == '.' {  
      let (i2, tail) = context("Decimal digit 0 - 9", parse_dec_digits)(i2)?;
      return Ok((i2, format!("{}.{}", head, tail)));
    }
  }
  
  Ok((i, String::from(*head)))
}
  
fn parse_symbol(i: Span) -> IResult<Span, String> {
  let (i, head) = 
    context("Alphabetic character", take_while1(|c: char| c.is_alphabetic()))(i)?;
  let (i, tail) = 
    take_while(|c: char| c.is_alphanumeric() || c == '?')(i)?;
  
  Ok((i, format!("{}{}", head, tail)))
}

fn parse_typename(i: Span) -> IResult<Span, String> {
  let (i, head) =
    context("Uppercase alphabetic character", take_while1(|c: char| c.is_alphabetic() && c.is_uppercase()))(i)?;
  let (i, tail) =
    take_while(|c: char| c.is_alphanumeric())(i)?;

  Ok((i, format!("{}{}", head, tail)))
}

fn parse_atomic(i: Span) -> IResult<Span, Expr> {
  let pos = position(i)?.0;
  let (_, c) = anychar(i)?;
  
  if c.is_alphabetic() {
    parse_symbol(i)
      .map(|r| (r.0, Expr {
        position: pos,
        kind: ExprKind::Symbol(r.1)
      }))
  } else if c.is_digit(10) {
    let (i, f) = parse_dec_float(i)?;
    let n = f.parse::<f64>().map_err(|_| gen_verbose(i, "Valid float"))?;
    Ok((i, Expr {
      position: pos,
      kind: ExprKind::Number(n)
    }))
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

fn parse_binary(i: Span, min_bp: u8) -> IResult<Span, Expr> {
  let (mut i, mut lhs) = parse_atomic(i)?;
  
  loop {
    let i2 = nlspace0(i)?.0;
    let (i2, op) = take_while(|c: char| infix_bp(c).is_some())(i2)?;
    if *op == "" { break; }
    
    if let Some((l_bp, r_bp)) = infix_bp(op.chars().next().unwrap()) {
      if l_bp < min_bp {
        break;
      }
      i = nlspace0(i2)?.0;
      
      let rhs = parse_binary(i, r_bp)?;
      i = rhs.0;
      lhs = Expr {
        position: i, 
        kind: ExprKind::Binary(Box::new(lhs), op.to_string(), Box::new(rhs.1))
      };
    } else {
      return Err(gen_verbose(op, "Valid infix operator"))
    }

  }
  Ok((i, lhs))
}

fn parse_parameters(i: Span) -> IResult<Span, Vec<(String, String)>> {
  let mut result = vec![];
  
  let mut i2 = i;

  loop {
    let namet = context("Symbol", parse_symbol)(i2)?;
    i2 = nlspace0(namet.0)?.0;
    let typet = context("Type name", parse_typename)(i2)?;
    let (i3, peeked) = peek_next(typet.0)?;
    if peeked == ',' {
      i2 = nlspace0(i3)?.0;
      result.push((namet.1, typet.1));
      continue;
    } else {
      result.push((namet.1, typet.1));
      break;
    }
  }

  Ok((i2, result))
}

fn parse_stmt_funcdef(i: Span) -> IResult<Span, Expr> {
  let (pos, _) = position(i)?;
  let (i, name) = many1(
    context( "Namespace or function name",
    alt((
        terminated(parse_typename, tag("::")),
        parse_symbol,
      ))
    )
  )(i)?;

  let (i, params) = opt(preceded(
    char('('),
    char(')').map(|_| None).or(
      terminated( parse_parameters, char(')')).map(|r| Some(r))
    )
  ))(i)?;
  
  let (i, returntype) = opt(preceded(
    tag("->"),
    context("Return type", parse_typename)
  ))(i)?;

  let (i, expr) = preceded(
    newline,
    tag("end").map(|r| Expr { position: r, kind: ExprKind::Null }).or(
      terminated(parse_stmt_block, tag("end"))
    )
  )(i)?;
  
  let params = match params {
    Some(o) => o,
    None => None,
  };

  Ok((i, Expr {
    position: pos,
    kind: ExprKind::FuncDef(name, params, returntype, Box::new(expr))
  }))
}

fn parse_stmt(i: Span) -> IResult<Span, Expr> {
  let (i2, keywordopt) = terminated(opt(parse_symbol), nlspace0)(i)?;
  if let Some(keyword) = keywordopt {
    match keyword.as_str() {
      "def" => parse_stmt_funcdef(i2),
      _ => parse_binary(i, 0),
    }
  } else {
    parse_binary(i, 0)
  }
}

fn parse_stmt_block(i: Span) -> IResult<Span, Expr> {
  let (i2, block) = many1(
    terminated(
      parse_stmt,
      nlspace1,
    )
  )(i)?;

  Ok((i2, Expr {
    position: i,
    kind: ExprKind::Block(block)
  }))
}

fn expr(i: Span) -> IResult<Span, Expr> {
  parse_stmt(i)
  // Ok((i, Expr {
  //   position: i,
  //   kind: ExprKind::Symbol(format!("{:?}", parse_opttyped_parameters(i).unwrap()))
  // }))
}

pub fn parse(src: &str) -> IResult<Span, Expr> {
  expr(Span::new(src))
}