extern crate nom;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
extern crate regex;
#[macro_use]
extern crate clap;

use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::Path;

mod ast;
mod index_comb;
mod parser;
mod tables;

fn find_files(p: &Path) -> Vec<(File, String)> {
  fs::read_dir(p)
    .unwrap()
    .map(|res| res.unwrap().path())
    .filter(|e| e.extension() == Some(OsStr::new("csv")))
    .map(|p| {
      (
        File::open(&p).unwrap(),
        p.file_stem().unwrap().to_str().unwrap().to_string(),
      )
    })
    .collect::<Vec<_>>()
}

fn main() -> Result<(), String> {
  let matches = clap_app!(morsql =>
      (version: "1.0")
      (author: "Lucas Silver and Harry Goldstein")
      (about: "A small DBMS.")
      (@arg INPUT: +required "The query file to run.")
      (@arg data_dir: -d --data +takes_value "The directory containing the data CSV files.")
  )
  .get_matches();

  let query_file = matches.value_of("INPUT").unwrap();
  let query_text = fs::read_to_string(query_file).map_err(|e| e.to_string())?;
  let query = parser::parse_sql(&query_text)?;

  let (files, names) = find_files(
    matches
      .value_of("data_dir")
      .map(|s| Path::new(s))
      .unwrap_or(env::current_dir().unwrap().as_path()),
  )
  .into_iter()
  .unzip();
  let data = query.run_from_files(&files, &names)?;

  println!("{}", data);

  Ok(())
}
