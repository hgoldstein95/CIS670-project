extern crate nom;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod ast;
mod parser;

fn main() {
  println!("Hello, world!");
}
