extern crate nom;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
extern crate regex;

use std::env;
use std::fs;
use std::fs::File;

mod ast;
mod index_comb;
mod parser;
mod tables;

fn main() -> Result<(), String> {
  let args: Vec<String> = env::args().collect();
  let query_file = &args[1];
  let query_text = fs::read_to_string(query_file).map_err(|e| e.to_string())?;
  let query = parser::parse_sql(&query_text)?;

  let data = query.run_from_files(
    &vec![File::open("examples/user.csv").map_err(|e| e.to_string())?],
    &vec!["user".to_string()],
  )?;

  println!("{}", data);

  Ok(())
}
