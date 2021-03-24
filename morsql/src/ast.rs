#![allow(dead_code)]

pub type Ident = String;
pub type Table = String;

#[derive(Debug, PartialEq)]
pub struct Query {
  selection: Selection,
  tables: Vec<Table>,
  filter: Filter,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Selection {
  Star,
  Columns(Vec<Ident>),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOp {
  Neg,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOp {
  And,
  Or,
  Eq,
  Lt,
  Like,
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
