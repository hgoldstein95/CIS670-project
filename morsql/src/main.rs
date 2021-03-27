extern crate nom;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use std::env;
use std::error;
use std::fs;

mod ast;
mod parser;

fn main() -> Result<(), Box<dyn error::Error + 'static>> {
  let args: Vec<String> = env::args().collect();
  let query_file = &args[1];
  let query_text = fs::read_to_string(query_file)?;

  let query = parser::parse_sql(&query_text)?;

  // TODO: Run query
  println!("{}", query);

  Ok(())
}
