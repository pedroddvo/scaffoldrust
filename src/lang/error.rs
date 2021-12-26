use nom::error::*;
use VerboseErrorKind::*;
use nom_locate::LocatedSpan;
use std::fmt::Write;

pub fn report_span(src: &str, span: LocatedSpan<&str>, reason: String) -> String {
  let mut result = String::new();
  let line_number = span.location_line();
  let offset = span.location_offset();
  let column = span.get_column();
  
  // let line_begin = src.as_bytes()[..offset].iter().rev().position(|c| *c == b'\n').map(|u| u).unwrap_or(0);
  // let line_end = src.as_bytes()[line_begin..].iter().position(|c| *c == b'\n').unwrap_or(src.len());
  

  write!(&mut result, "{line_number}:{column}: error: {reason}",
    line_number=line_number,
    column=column,
    reason=reason,
  ).unwrap();
  

  // write!(&mut result, "\t{}\n", &src[line_begin..line_end]).unwrap();
  // write!(&mut result, "\t{}^\n", " ".repeat(column-1)).unwrap();
  result
}

pub fn report_error(src: &str, verr: VerboseError<LocatedSpan<&str>>) -> String {

  let mut result = String::new();
  println!("{:#?}", verr.errors);

  for (i, (slice, kind)) in verr.errors.iter().enumerate() {
    let mut reason = String::new();

    match kind {
      Nom(ekind) => {
        match ekind {
          ErrorKind::Eof => {
            write!(&mut reason, "Unexpected end of file!\n").unwrap();
          }
          _ => continue
        }
      }
      Context(ctx) => {
        write!(&mut reason, "Expected {:?}\n", ctx).unwrap();
      }
      Char(c) => {
        write!(&mut reason, "Expected character {:?}, got {:?}", c, slice.chars().nth(slice.get_column()).unwrap()).unwrap();
      }
      _ => continue
    }

    write!(&mut result, "{}\n\n", report_span(src, *slice, reason)).unwrap();
  }
  
  result
}