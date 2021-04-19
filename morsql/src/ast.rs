pub type Ident = String;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOp {
  Not,
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
  Id(ColumnSelector),
  LitS(String),
  LitB(bool),
  LitI(i64),
  UnaryOp(UnaryOp, Box<Filter>),
  BinaryOp(BinaryOp, Box<Filter>, Box<Filter>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ColumnSelector {
  pub table: Option<Ident>,
  pub field: Ident,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IndexedFilter {
  Id(IndexedColumnSelector),
  LitS(String),
  LitB(bool),
  LitI(i64),
  UnaryOp(UnaryOp, Box<IndexedFilter>),
  BinaryOp(BinaryOp, Box<IndexedFilter>, Box<IndexedFilter>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IndexedColumnSelector {
  pub table : usize,
  pub field : usize
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Selection {
  Star,
  Columns(Vec<ColumnSelector>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Table {
  pub table_name: Ident,
  pub alias: Option<Ident>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Query {
  pub selection: Selection,
  pub tables: Vec<Table>,
  pub filter: Filter,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IndexedQuery {
  pub selection: Selection,
  pub tables: Vec<Table>,
  pub filter: IndexedFilter,
}

mod display {
  use super::*;
  use std::fmt;

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

  impl fmt::Display for ColumnSelector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(
        f,
        "{}{}",
        match &self.table {
          None => "".to_owned(),
          Some(x) => format!("{}.", x),
        },
        self.field
      )
    }
  }

  impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      match self {
        Selection::Star => write!(f, "*"),
        Selection::Columns(ts) => write!(
          f,
          "{}",
          ts.iter()
            .map(|t| format!("{}", t))
            .collect::<Vec<_>>()
            .join(", "),
        ),
      }
    }
  }

  impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(
        f,
        "{}{}",
        self.table_name,
        match &self.alias {
          None => "".to_owned(),
          Some(x) => format!(" AS {}", x),
        }
      )
    }
  }

  impl fmt::Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(
        f,
        "SELECT {}\nFROM {}\nWHERE {}",
        self.selection,
        self
          .tables
          .iter()
          .map(|t| format!("{}", t))
          .collect::<Vec<_>>()
          .join(", "),
        self.filter
      )
    }
  }
}

#[cfg(test)]
mod generators {
  use super::*;
  use quickcheck::Arbitrary;
  use quickcheck::Gen;

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
      0 => Filter::Id(ColumnSelector::arbitrary(g)),
      1 => Filter::LitS(gen_string_lit(g)),
      // TODO: Known issue, our parser doesn't handle huge things well
      2 => Filter::LitI(i32::arbitrary(g) as i64),
      3 => Filter::LitB(bool::arbitrary(g)),
      _ => unreachable!(),
    }
  }

  impl Arbitrary for UnaryOp {
    fn arbitrary(_g: &mut Gen) -> Self {
      UnaryOp::Not
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

  impl Arbitrary for ColumnSelector {
    fn arbitrary(g: &mut Gen) -> Self {
      ColumnSelector {
        table: if bool::arbitrary(g) {
          None
        } else {
          Some(gen_ident(g))
        },
        field: gen_ident(g),
      }
    }
  }

  impl Arbitrary for Selection {
    fn arbitrary(g: &mut Gen) -> Self {
      if bool::arbitrary(g) {
        Selection::Star
      } else {
        let mut v = Vec::arbitrary(g);
        v.push(ColumnSelector::arbitrary(g));
        Selection::Columns(v)
      }
    }
  }

  impl Arbitrary for Table {
    fn arbitrary(g: &mut Gen) -> Self {
      Table {
        table_name: gen_ident(g),
        alias: if bool::arbitrary(g) {
          None
        } else {
          Some(gen_ident(g))
        },
      }
    }
  }

  impl Arbitrary for Query {
    fn arbitrary(g: &mut Gen) -> Self {
      let mut v = Vec::arbitrary(g);
      v.push(Table::arbitrary(g));
      Query {
        selection: Selection::arbitrary(g),
        tables: v,
        filter: Filter::arbitrary(g),
      }
    }
  }
}
