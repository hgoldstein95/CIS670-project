#![allow(dead_code)]

use std::fs::File;

pub type Ident = String;
pub type Table = File;

pub struct Query {
  selection: Selection,
  tables: Vec<Table>,
  filter: Filter,
}

pub enum Selection {
  Star,
  Columns(Vec<Ident>),
}

#[derive(Debug)]
pub enum Filter {
  Id(Ident),
  LitS(String),
  LitB(bool),
  LitI(i64),
  And(Box<Filter>, Box<Filter>),
  Or(Box<Filter>, Box<Filter>),
  Not(Box<Filter>),
  Eq(Box<Filter>, Box<Filter>),
  Lt(Box<Filter>, Box<Filter>),
  Like(Box<Filter>, Box<Filter>),
}