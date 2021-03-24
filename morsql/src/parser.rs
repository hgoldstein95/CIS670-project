#![allow(dead_code)]

use crate::ast::*;
use nom::branch::alt;
use nom::bytes::complete::escaped;
use nom::bytes::complete::is_not;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::*;
use nom::combinator::map;
use nom::combinator::opt;
use nom::error::{context, VerboseError};
use nom::multi::fold_many0;
use nom::multi::separated_list1;
use nom::sequence::pair;
use nom::sequence::terminated;
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

fn p_int(input: &str) -> Res<&str, i64> {
  let (input, s) = opt(tag("-"))(input)?;
  let (input, ds) = digit1(input)?;
  let n = ds.parse::<i64>().expect("failed to parse difits");
  Ok((input, if s.is_some() { -n } else { n }))
}

fn p_bool(input: &str) -> Res<&str, bool> {
  context(
    "bool",
    alt((
      map(tag_no_case("TRUE"), |_| true),
      map(tag_no_case("FALSE"), |_| false),
    )),
  )(input)
}

fn p_string(input: &str) -> Res<&str, String> {
  context(
    "string",
    alt((
      map(tag("\"\""), |_| "".to_owned()),
      map(
        delimited(
          char('"'),
          escaped(is_not("\""), '\\', one_of(r#""n\"#)),
          char('"'),
        ),
        String::from,
      ),
    )),
  )(input)
}

fn factor(input: &str) -> Res<&str, Filter> {
  fn not(i: &str) -> Res<&str, Filter> {
    let (i, _) = tag_no_case("NOT")(i)?;
    let (i, _) = space0(i)?;
    let (i, f) = factor(i)?;
    Ok((i, Filter::UnaryOp(UnaryOp::Not, Box::new(f))))
  }

  context(
    "factor",
    alt((
      delimited(char('('), expression, char(')')),
      not,
      map(p_int, Filter::LitI),
      map(p_bool, Filter::LitB),
      map(p_string, Filter::LitS),
      map(ident, Filter::Id),
    )),
  )(input)
}

fn term(input: &str) -> Res<&str, Filter> {
  let (input, init) = factor(input)?;
  let (input, _) = space0(input)?;

  context(
    "term",
    fold_many0(
      pair(
        alt((
          terminated(alt((tag("&&"), tag("=="), tag("<"))), space0),
          terminated(tag_no_case("LIKE"), space0),
        )),
        factor,
      ),
      init,
      |acc: Filter, (op, f): (&str, Filter)| match op.to_lowercase().as_str() {
        "&&" => Filter::BinaryOp(BinaryOp::And, Box::new(acc), Box::new(f)),
        "==" => Filter::BinaryOp(BinaryOp::Eq, Box::new(acc), Box::new(f)),
        "<" => Filter::BinaryOp(BinaryOp::Lt, Box::new(acc), Box::new(f)),
        "like" => Filter::BinaryOp(BinaryOp::Like, Box::new(acc), Box::new(f)),
        _ => panic!("invalid term op"),
      },
    ),
  )(input)
}

fn expression(input: &str) -> Res<&str, Filter> {
  let (input, init) = term(input)?;
  let (input, _) = space0(input)?;

  context(
    "expression",
    fold_many0(
      pair(terminated(tag("||"), space0), term),
      init,
      |acc: Filter, (op, f): (&str, Filter)| match op {
        "||" => Filter::BinaryOp(BinaryOp::Or, Box::new(acc), Box::new(f)),
        _ => panic!("invalid factor op"),
      },
    ),
  )(input)
}

fn selection(input: &str) -> Res<&str, Selection> {
  context(
    "selection",
    alt((
      map(tag("*"), |_| Selection::Star),
      map(separated_list1(terminated(tag(","), space0), ident), |xs| {
        Selection::Columns(xs)
      }),
    )),
  )(input)
}

fn query(input: &str) -> Res<&str, Query> {
  let (input, _) = tag_no_case("SELECT")(input)?;
  let (input, _) = multispace1(input)?;
  let (input, s) = selection(input)?;
  let (input, _) = multispace1(input)?;
  let (input, _) = tag_no_case("FROM")(input)?;
  let (input, _) = multispace1(input)?;
  let (input, t) = separated_list1(terminated(tag(","), space0), ident)(input)?;
  let (input, _) = multispace1(input)?;
  let (input, _) = tag_no_case("WHERE")(input)?;
  let (input, _) = space1(input)?;
  let (input, f) = expression(input)?;

  Ok((input, Query::new(s, t, f)))
}

#[cfg(test)]
mod tests {

  use super::*;

  #[quickcheck]
  fn prop_round_trip(f: Filter) -> bool {
    f == expression(&format!("{}", f)).unwrap().1
  }

  #[test]
  fn strings_work() {
    assert_eq!(p_string("\"foo\""), Ok(("", "foo".to_owned())));
    assert_eq!(p_string("\"\""), Ok(("", "".to_owned())));
  }

  #[test]
  fn whole_query() {
    assert_eq!(
      query("SELECT name, id\nFROM users\nWHERE name == \"Harry\""),
      Ok((
        "",
        Query::new(
          Selection::Columns(vec!["name".to_owned(), "id".to_owned()]),
          vec!["users".to_owned()],
          Filter::BinaryOp(
            BinaryOp::Eq,
            Box::new(Filter::Id("name".to_owned())),
            Box::new(Filter::LitS("Harry".to_owned()))
          )
        )
      ))
    )
  }

  #[test]
  fn filter_not() {
    assert_eq!(
      expression("NOT x && y"),
      Ok((
        "",
        Filter::BinaryOp(
          BinaryOp::And,
          Box::new(Filter::UnaryOp(
            UnaryOp::Not,
            Box::new(Filter::Id("x".to_owned()))
          )),
          Box::new(Filter::Id("y".to_owned()))
        )
      ))
    );
  }

  #[test]
  fn filter_and_or() {
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
      expression("(x && y) && z"),
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
      expression("x&&  (y&&z)"),
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
