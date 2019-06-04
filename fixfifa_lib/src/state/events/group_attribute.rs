use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use std::string::ToString;

#[derive(Debug, PartialEq)]
pub struct GroupAttribute {
  /// (club-) captain
  c: String,
  /// (club-) id
  i: String,
  /// (club-) abbreviation
  o: String,
  /// (club-) name
  n: String,
  z: String,
  t: String,
  j: String,
}

///
/// idea from: https://rust-lang-nursery.github.io/rust-cookbook/text/string_parsing.html
///
impl GroupAttribute {
  /// https://users.rust-lang.org/t/idiomatic-way-to-construct-object-with-some-non-required-fields/8078/4
  pub fn new() -> Self {
    /// https://doc.rust-lang.org/std/string/struct.String.html#method.new
    GroupAttribute {
      i: "".to_string(),
      o: "".to_string(),
      n: "".to_string(),
      z: "".to_string(),
      t: "".to_string(),
      j: "".to_string(),
      c: "".to_string(),
    }
  }
}

impl FromStr for GroupAttribute {
  type Err = std::num::ParseIntError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    //
    let parts: Vec<&str> = s.split_terminator(';').collect();
    let mut parts_iter = parts.into_iter();

    /// https://doc.rust-lang.org/rust-by-example/fn/closures/closure_examples/iter_find.html
    let i = parts_iter.find(|&p| p.starts_with("I_")).unwrap();
    let o = parts_iter.find(|&p| p.starts_with("O_")).unwrap();
    let n = parts_iter.find(|&p| p.starts_with("N_")).unwrap();
    let z = parts_iter.find(|&p| p.starts_with("Z_")).unwrap();
    let t = parts_iter.find(|&p| p.starts_with("T_")).unwrap();
    let j = parts_iter.find(|&p| p.starts_with("J_")).unwrap();
    let c = parts_iter.find(|&p| p.starts_with("C_")).unwrap();

    Ok(GroupAttribute {
      i: String::from_str(o).unwrap(),
      o: String::from_str(o).unwrap(),
      n: String::from_str(n).unwrap(),
      z: String::from_str(z).unwrap(),
      t: String::from_str(t).unwrap(),
      j: String::from_str(j).unwrap(),
      c: String::from_str(c).unwrap(),
    })
  }
}
