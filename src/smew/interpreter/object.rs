use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
  Number(f64),
  Str(String),
  Bool(bool),
  Record(Record),
  Nil,
}

impl Object {
  pub fn to_str_object(&self) -> Option<Self> {
    use self::Object::*;

    let result = match *self {
      Number(ref a) => Str(a.to_string()),
      Bool(ref a)   => Str(a.to_string()),
      Str(ref a)    => Str(a.clone()),
      Nil           => Str(String::from("<nil>")),
      _             => return None,
    };

    Some(result)
  }
}

impl fmt::Display for Object {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use self::Object::*;

    match *self {
      Str(ref content) => write!(f, "{}", content),
      _                => Ok(()),
    }
  }
}



#[derive(Debug, Clone, PartialEq)]
pub struct Record {
  pub content: Vec<Object>,
  pub map:     HashMap<String, Record>,
}

impl Record {
  pub fn new(content: Vec<Object>, map: HashMap<String, Record>) -> Self {
    Record {
      content,
      map,
    }
  }
}