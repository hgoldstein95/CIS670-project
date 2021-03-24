#![allow(dead_code)]

use std::fmt;

pub type Ident = String;
pub type Table = String;

#[derive(Debug, PartialEq)]
pub struct Query {
  selection: Selection,
  tables: Vec<Table>,
  filter: Filter,
}

impl Query {
  pub fn new(s: Selection, t: Vec<Table>, f: Filter) -> Self {
    Query {
      selection: s,
      tables: t,
      filter: f,
    }
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Selection {
  Star,
  Columns(Vec<Ident>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOp {
  Not,
}

impl fmt::Display for UnaryOp {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        UnaryOp::Not => "NOT",
      }
    )
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOp {
  And,
  Or,
  Eq,
  Lt,
  Like,
}

impl fmt::Display for BinaryOp {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::Eq => "==",
        BinaryOp::Lt => "<",
        BinaryOp::Like => "LIKE",
      }
    )
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Filter {
  Id(Ident),
  LitS(String),
  LitB(bool),
  LitI(i64),
  UnaryOp(UnaryOp, Box<Filter>),
  BinaryOp(BinaryOp, Box<Filter>, Box<Filter>),
}

impl fmt::Display for Filter {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Filter::Id(i) => write!(f, "{}", i),
      Filter::LitS(s) => write!(f, "\"{}\"", s),
      Filter::LitB(b) => write!(f, "{}", b),
      Filter::LitI(n) => write!(f, "{}", n),
      Filter::UnaryOp(op, e) => write!(f, "({} {})", op, e),
      Filter::BinaryOp(op, e1, e2) => write!(f, "({} {} {})", e1, op, e2),
    }
  }
}
