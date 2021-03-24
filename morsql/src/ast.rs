#![allow(dead_code)]

use quickcheck::Arbitrary;
use quickcheck::Gen;
use std::fmt;

pub type Ident = String;
pub type Table = String;

#[derive(Debug, PartialEq, Clone)]
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

impl Arbitrary for Query {
  fn arbitrary(g: &mut Gen) -> Self {
    Query::new(
      Selection::arbitrary(g),
      Vec::arbitrary(g),
      Filter::arbitrary(g),
    )
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Selection {
  Star,
  Columns(Vec<Ident>),
}

impl Arbitrary for Selection {
  fn arbitrary(g: &mut Gen) -> Self {
    if bool::arbitrary(g) {
      Selection::Star
    } else {
      Selection::Columns(Vec::arbitrary(g))
    }
  }
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

impl Arbitrary for UnaryOp {
  fn arbitrary(_g: &mut Gen) -> Self {
    UnaryOp::Not
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

impl Arbitrary for BinaryOp {
  fn arbitrary(g: &mut Gen) -> Self {
    *g.choose(&[
      BinaryOp::And,
      BinaryOp::Or,
      BinaryOp::Eq,
      BinaryOp::Lt,
      BinaryOp::Like,
    ])
    .unwrap()
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
      Filter::LitS(s) => write!(f, r#""{}""#, s),
      Filter::LitB(b) => write!(f, "{}", b),
      Filter::LitI(n) => write!(f, "{}", n),
      Filter::UnaryOp(op, e) => write!(f, "({} {})", op, e),
      Filter::BinaryOp(op, e1, e2) => write!(f, "({} {} {})", e1, op, e2),
    }
  }
}

fn gen_alpha(g: &mut Gen) -> char {
  *g.choose(&('a'..='z').collect::<Vec<char>>()).unwrap()
}

fn gen_list<T, F: Fn(&mut Gen) -> T>(g: &mut Gen, f: F) -> Vec<T> {
  let n = *g.choose(&(0..=g.size()).collect::<Vec<_>>()).unwrap();
  let mut v = vec![];
  for _ in 0..n {
    v.push(f(g));
  }
  v
}

fn gen_string<F: Fn(&mut Gen) -> char>(g: &mut Gen, f: F) -> String {
  gen_list(g, f).into_iter().collect()
}

fn gen_ident(g: &mut Gen) -> String {
  loop {
    let s = gen_string(g, gen_alpha);
    if !s.is_empty() {
      return s;
    }
  }
}

fn gen_string_lit(g: &mut Gen) -> String {
  gen_string(g, gen_alpha) // TODO: This should be more interesting
}

fn gen_lit(g: &mut Gen) -> Filter {
  match g.choose(&[0, 1, 2, 3]).unwrap() {
    0 => Filter::Id(gen_ident(g)),
    1 => Filter::LitS(gen_string_lit(g)),
    // TODO: Known issue, our parser doesn't handle huge things well
    2 => Filter::LitI(i32::arbitrary(g) as i64),
    3 => Filter::LitB(bool::arbitrary(g)),
    _ => unreachable!(),
  }
}

impl Arbitrary for Filter {
  fn arbitrary(g: &mut Gen) -> Self {
    let n = g.size();
    if n <= 1 {
      gen_lit(g)
    } else {
      match g.choose(&[0, 1, 2, 3, 4, 5]).unwrap() {
        0..=3 => gen_lit(g),
        4 => Filter::UnaryOp(
          UnaryOp::arbitrary(g),
          Box::new(Filter::arbitrary(&mut Gen::new(n - 1))),
        ),
        5 => Filter::BinaryOp(
          BinaryOp::arbitrary(g),
          Box::new(Filter::arbitrary(&mut Gen::new(n / 2))),
          Box::new(Filter::arbitrary(&mut Gen::new(n / 2))),
        ),
        _ => unreachable!(),
      }
    }
  }
}
