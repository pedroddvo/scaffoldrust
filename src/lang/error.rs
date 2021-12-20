

#[derive(Clone)]
pub struct LangError {
  msg: String,
  
  start: isize,
  offset: isize,
}

pub struct LangTrace {
  errors: [Option<LangError>; 8],
  error_point: usize,
  error_count: usize,
}

impl LangTrace {
  pub fn new() -> Self {
    Self { errors: Default::default(), error_point: 0, error_count: 0 }
  }

  pub fn errored(&self) -> bool {
    return self.error_count >= 1;
  }
  
  pub fn error(&mut self, start: usize, offset: isize, msg: String) {
    if self.error_point == 8 {
      self.error_point = 0;
    }
    
    self.errors[self.error_point] = Some(LangError { start, offset, msg });
  }
  
  pub fn report(&self, src: &str) {
    for err_op in &self.errors {
      if let Some(err) = err_op {
        let line_n = src[0..(err.start + err.offset)];
      }
    }
  }
}