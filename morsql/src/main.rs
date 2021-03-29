extern crate nom;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

extern crate regex;

mod ast;
mod parser;
mod tables;
mod index_comb;


fn main() -> Result<(), String> {
  let query = parser::parse_sql("SELECT id, name FROM users where name == \"Harry\"")?;
  println!("{}", query);
  Ok(())
}
