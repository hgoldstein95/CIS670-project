extern crate nom;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod ast;
mod parser;

fn main() -> Result<(), String> {
  let query = parser::parse_sql("SELECT id, name FROM users where name == \"Harry\"")?;
  println!("{}", query);
  Ok(())
}
