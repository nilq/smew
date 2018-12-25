use std::collections::HashMap;

use super::super::error::Response::Wrong;
use super::*;

pub struct Frame {
  pub locals: HashMap<String, Object>,
}

impl Frame {
  pub fn new() -> Self {
    Frame {
      locals: HashMap::new()
    }
  }

  pub fn set_name(&mut self, name: &String, value: Object) {
    self.locals.insert(name.clone(), value);
  }

  pub fn find_name(&self, name: &str) -> Option<&Object> {
    self.locals.get(name)
  }
}


pub struct Interpreter<'a> {
  stack: Vec<Frame>,
  source: &'a Source,
}

impl<'a> Interpreter<'a> {
  pub fn new(source: &'a Source) -> Self {
    Interpreter {
      stack: vec!(Frame::new()),
      source,
    }
  }



  pub fn evaluate(&mut self, ast: &Vec<Statement>) -> Result<Record, ()> {
    use self::StatementNode::*;

    let mut content = Vec::new();
    let mut map     = HashMap::new();

    for statement in ast.iter() {
      match statement.node {
        self::StatementNode::Record(ref name, ref parents, ref body) => {
          self.stack.push(Frame::new());

          let record = self.evaluate(body)?;

          map.insert(name.to_owned(), record.clone());

          self.set_binding(name, Object::Record(record));
          self.stack.pop();
        },

        Assignment(ref name, ref right) => {
          let right = self.evaluate_expression(right)?;

          self.set_binding(name, right);
        },

        Expression(ref expression) => {
          let expression = self.evaluate_expression(expression)?;

          content.push(expression)
        }

        _ => (),
      }
    }

    Ok(self::Record::new(content, map))
  }

  pub fn evaluate_expression(&mut self, expression: &Expression) -> Result<Object, ()> {
    use self::ExpressionNode::*;

    let result = match expression.node {
      Number(ref n) => Object::Number(*n),
      Str(ref n)    => Object::Str(n.clone()),
      Bool(ref n)   => Object::Bool(*n),
      Identifier(ref n) => {
        self.find_name(n)?
      },

      Neg(ref expression) => {
        let value = self.evaluate_expression(expression)?;

        if let Object::Number(ref a) = value {
          Object::Number(-a)
        } else {
          return Err(
            response!(
              Wrong("can't negate non-number"),
              self.source.file,
              expression.pos
            )
          )
        }
      },

      Not(ref expression) => {
        let value = self.evaluate_expression(expression)?;

        if let Object::Bool(ref a) = value {
          Object::Bool(!a)
        } else {
          return Err(
            response!(
              Wrong("can't flip non-boolean"),
              self.source.file,
              expression.pos
            )
          )
        }
      },

      Binary(ref a, ref op, ref b) => {
        use self::Operator::*;
        use self::Object::*;

        let a_value = self.evaluate_expression(&a)?;
        let b_value = self.evaluate_expression(&b)?;

        match (&a_value, op, &b_value) {
          (&Number(ref a), Add, &Number(ref b))  => Object::Number(a + b),
          (&Number(ref a), Sub, &Number(ref b))  => Object::Number(a - b),
          (&Number(ref a), Mul, &Number(ref b))  => Object::Number(a * b),
          (&Number(ref a), Div, &Number(ref b))  => Object::Number(a / b),
          (&Number(ref a), Lt, &Number(ref b))   => Object::Bool(a < b),
          (&Number(ref a), Gt, &Number(ref b))   => Object::Bool(a > b),
          (&Number(ref a), LtEq, &Number(ref b)) => Object::Bool(a <= b),
          (&Number(ref a), GtEq, &Number(ref b)) => Object::Bool(a >= b),

          (&Bool(a), And, &Bool(b)) => Object::Bool(a && b),
          (&Bool(a), Or, &Bool(b))  => Object::Bool(a || b),

          (ref a, Eq, ref b)   => Object::Bool(a == b),
          (ref a, NEq, ref b)  => Object::Bool(a != b),

          (&Record(ref record), Index, &Str(ref index)) => {
            if let Some(ref object) = record.map.get(index) {
              Object::Record((**object).clone())
            } else {
              return Err(
                response!(
                  Wrong(format!("no such field `{}` on record", index)),
                  self.source.file,
                  expression.pos
                )
              )
            }
          },

          (ref a, Concat, ref b) => {
            
            if let Some(ref a) = a.to_str_object() {
              if let Some(ref b) = b.to_str_object() {
                return Ok(Object::Str(format!("{}{}", a, b)))
              }
            }

            return Err(
              response!(
                Wrong(format!("can't perform operation `{:?}{}{:?}`", a, op, b)),
                self.source.file,
                expression.pos
              )
            )
          }

          _ => return Err(
            response!(
              Wrong(format!("can't perform operation `{:?}{}{:?}`", a, op, b)),
              self.source.file,
              expression.pos

            )
          )
        }
      },

      _ => Object::Nil,
    };

    Ok(result)
  }



  fn current_frame_mut(&mut self) -> &mut Frame {
    self.stack.last_mut().unwrap()
  }

  fn current_frame(&self) -> &Frame {
    self.stack.last().unwrap()
  }



  fn set_binding(&mut self, name: &String, value: Object) {
    self.current_frame_mut().set_name(name, value)
  }

  fn find_name(&self, name: &str) -> Result<Object, ()> {
    let mut parent_offset = self.stack.len() - 1;
    
    loop {
      if let Some(ref object) = self.stack[parent_offset].find_name(name) {
        return Ok((**object).clone())
      } else {
        if parent_offset == 0 {
          return Err(
            response!(
              Wrong(format!("no such thing as `{}`", name)),
              self.source.file
            )
          )
        }

        parent_offset -= 1;
      }
    }
  }
}