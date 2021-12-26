use nom::error::*;
use VerboseErrorKind::*;

fn slice_range(src_buf: &str, slice: &str) -> (usize, usize) {
  let start = slice.as_ptr() as usize - src_buf.as_ptr() as usize;
  let end = start + slice.len();
  (start, end)
}

pub fn report_error(src: &str, verr: VerboseError<&str>) -> String {
  use std::fmt::Write;

  let mut result = String::new();

  for (i, (slice, kind)) in verr.errors.iter().enumerate() {
    let (offs_start, offs_end) = slice_range(src, slice); 
    let prefix = &src.as_bytes()[..offs_start];
    
    let line_number = prefix.iter().filter(|c| **c == b'\n').count() + 1;
    let line_begin = prefix.iter().rev().position(|c| *c == b'\n').map(|pos| offs_start - pos).unwrap_or(0);
    let line_end = src.as_bytes()[line_begin..].iter().position(|c| *c == b'\n').unwrap_or(src.len());
    
    let column = offs_start - line_begin;
    
    
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
      _ => continue
    }

    write!(&mut result, "{line_number}:{column}: error: {reason}",
      line_number=line_number,
      column=column,
      reason=reason,
    ).unwrap();
    
  
    write!(&mut result, "\t{}\n", &src[line_begin..line_end]).unwrap();
    write!(&mut result, "\t{}^\n", " ".repeat(column)).unwrap();
  }
  
  result
}