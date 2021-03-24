#![allow(dead_code)]

use crate::ast::*;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case};
use nom::character::complete::*;
use nom::combinator::map;
use nom::error::{context, VerboseError};
use nom::multi::fold_many0;
use nom::sequence::pair;
use nom::sequence::{delimited, tuple};
use nom::IResult;

type Res<T, U> = IResult<T, U, VerboseError<T>>;

fn ident(input: &str) -> Res<&str, String> {
  context(
    "ident",
    map(tuple((alpha1, alphanumeric0)), |(x, y): (&str, &str)| {
      format!("{}{}", x, y)
    }),
  )(input)
}

fn pbool(input: &str) -> Res<&str, bool> {
  context(
    "bool",
    alt((
      map(tag_no_case("TRUE"), |_| true),
      map(tag_no_case("FALSE"), |_| false),
    )),
  )(input)
}

fn pstring(input: &str) -> Res<&str, String> {
  context(
    "string",
    map(delimited(char('"'), is_not("\""), char('"')), String::from),
  )(input)
}

fn factor(input: &str) -> Res<&str, Filter> {
  fn not(i: &str) -> Res<&str, Filter> {
    let (i, _) = tag_no_case("NOT")(i)?;
    let (i, f) = factor(i)?;
    Ok((i, Filter::UnaryOp(UnaryOp::Neg, Box::new(f))))
  }

  context(
    "factor",
    alt((
      map(ident, Filter::Id),
      map(digit1, |x: &str| Filter::LitI(x.parse::<i64>().unwrap())),
      map(pbool, Filter::LitB),
      map(pstring, Filter::LitS),
      not,
      delimited(char('('), expression, char(')')),
    )),
  )(input)
}

fn term(input: &str) -> Res<&str, Filter> {
  let (input, init) = factor(input)?;
  fold_many0(
    pair(tag("&&"), factor),
    init,
    |acc: Filter, (op, f): (&str, Filter)| match op {
      "&&" => Filter::BinaryOp(BinaryOp::And, Box::new(acc), Box::new(f)),
      _ => panic!("invalid term op"),
    },
  )(input)
}

fn expression(input: &str) -> Res<&str, Filter> {
  let (input, init) = term(input)?;
  fold_many0(
    pair(tag("||"), term),
    init,
    |acc: Filter, (op, f): (&str, Filter)| match op {
      "||" => Filter::BinaryOp(BinaryOp::Or, Box::new(acc), Box::new(f)),
      _ => panic!("invalid factor op"),
    },
  )(input)
}

fn selection(_input: &str) -> Res<&str, Selection> {
  unimplemented!()
}

fn query(_input: &str) -> Res<&str, Query> {
  unimplemented!()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parse_filter() {
    let x = Filter::Id("x".to_owned());
    let y = Filter::Id("y".to_owned());
    let z = Filter::Id("z".to_owned());

    assert_eq!(
      expression("x||y"),
      Ok((
        "",
        Filter::BinaryOp(BinaryOp::Or, Box::new(x.clone()), Box::new(y.clone()))
      ))
    );

    assert_eq!(
      expression("x||y&&z"),
      Ok((
        "",
        Filter::BinaryOp(
          BinaryOp::Or,
          Box::new(x.clone()),
          Box::new(Filter::BinaryOp(
            BinaryOp::And,
            Box::new(y.clone()),
            Box::new(z.clone())
          ))
        )
      ))
    );

    assert_eq!(
      expression("x&&y||z"),
      Ok((
        "",
        Filter::BinaryOp(
          BinaryOp::Or,
          Box::new(Filter::BinaryOp(
            BinaryOp::And,
            Box::new(x.clone()),
            Box::new(y.clone())
          )),
          Box::new(z.clone())
        )
      ))
    );

    assert_eq!(
      expression("(x&&y)&&z"),
      Ok((
        "",
        Filter::BinaryOp(
          BinaryOp::And,
          Box::new(Filter::BinaryOp(
            BinaryOp::And,
            Box::new(x.clone()),
            Box::new(y.clone())
          )),
          Box::new(z.clone())
        )
      ))
    );

    assert_eq!(
      expression("x&&(y&&z)"),
      Ok((
        "",
        Filter::BinaryOp(
          BinaryOp::And,
          Box::new(x),
          Box::new(Filter::BinaryOp(BinaryOp::And, Box::new(y), Box::new(z))),
        )
      ))
    );
  }
}
