use super::parse::{Expr, parse, ExprKind};

#[derive(Debug)]
pub enum Node {
  Number(f64), 
}


pub struct Env {
  
}

impl Env {


  pub fn expr(&mut self, expr: Expr) -> Result<Node, String> {
    match expr.kind {
      ExprKind::Number(n) => Ok(Node::Number(n)),
      _ => unimplemented!()
    }
  }
}