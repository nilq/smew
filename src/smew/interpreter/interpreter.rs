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


pub type ForeignFunction = fn(&Vec<Object>) -> Object;

pub struct Interpreter<'a> {
  stack: Vec<Frame>,
  source: &'a Source,
  foreign: HashMap<String, ForeignFunction>,
}

impl<'a> Interpreter<'a> {
  pub fn new(source: &'a Source, foreign: HashMap<String, ForeignFunction>) -> Self {
    Interpreter {
      stack: vec!(Frame::new()),
      source,
      foreign
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

          let mut inherited_map = HashMap::new();

          for parent in parents {
            let parent_record = self.evaluate_expression(&parent)?;

            if let Object::Record(record) = parent_record {
              for (name, value) in record.map.iter() {

                inherited_map.insert(name.clone(), value.clone());
                
                let value = if value.map.len() == 0 && record.content.len() == 1 {
                  record.content[0].clone()
                } else {
                  Object::Record((*value).clone())
                };

                self.set_binding(name, value.clone());
              }
            } else {
              return Err(
                response!(
                  Wrong("can't inherit from non-record"),
                  self.source.file,
                  statement.pos
                )
              )
            }
          }

          let mut record = self.evaluate(body)?;

          record.map.extend(inherited_map);

          let value = if record.map.len() == 0 && record.content.len() == 1 {
            record.content[0].clone()
          } else {
            Object::Record(record.clone())
          };

          map.insert(name.to_owned(), record);
          self.stack.pop();

          self.set_binding(name, value);          
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
      Number(ref n)     => Object::Number(*n),
      Str(ref n)        => Object::Str(n.clone()),
      Bool(ref n)       => Object::Bool(*n),
      Identifier(ref n) => self.find_name(n, &expression.pos)?,

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

      Call(ref expression, ref args) => {
        let mut result     = Object::Nil;
        let mut arg_values = Vec::new();

        for arg in args {
          arg_values.push(self.evaluate_expression(arg)?)
        }
        
        if let ExpressionNode::Identifier(ref name) = expression.node {
          if let Some(func) = self.foreign.get(name) {

            result = func(&arg_values)
          }
        }

        result
      },

      Binary(ref a, ref op, ref b) => {
        use self::Operator::*;
        use self::Object::*;

        let a_value = self.evaluate_expression(&a)?;

        if let Record(ref record) = a_value {
          if *op == Index {
            if let ExpressionNode::Identifier(ref index) = b.node {
              if let Some(ref object) = record.map.get(index) {

                let record = (**object).clone();

                let result = if record.map.len() == 0 && record.content.len() == 1 {
                  record.content[0].clone()
                } else {
                  Object::Record(record)
                };

                return Ok(result)
              
              } else {
                return Err(
                  response!(
                    Wrong(format!("no such field `{}` on record", index)),
                    self.source.file,
                    expression.pos
                  )
                )
              }
            }
          }
        }

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

          (ref a, Concat, ref b) => {
            
            if let Some(ref a) = a.to_str_object() {
              if let Some(ref b) = b.to_str_object() {
                return Ok(Object::Str(format!("{}{}", a, b)))
              }
            }

            return Err(
              response!(
                Wrong(format!("can't perform operation `{:?}{}{:?}`", a_value, op, b_value)),
                self.source.file,
                expression.pos
              )
            )
          }

          _ => return Err(
            response!(
              Wrong(format!("can't perform operation `{:?}{}{:?}`", a_value, op, b_value)),
              self.source.file,
              expression.pos

            )
          )
        }
      },

      _ => Object::Nil,
    };

    if let Object::Record(ref record) = result {
      if record.map.len() == 0 && record.content.len() == 1 {
        return Ok(record.content[0].clone())
      }
    }

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

  fn find_name(&self, name: &str, pos: &Pos) -> Result<Object, ()> {
    let mut parent_offset = self.stack.len() - 1;
    
    loop {
      if let Some(ref object) = self.stack[parent_offset].find_name(name) {
        return Ok((**object).clone())
      } else {
        if parent_offset == 0 {
          return Err(
            response!(
              Wrong(format!("no such thing as `{}`", name)),
              self.source.file,
              pos
            )
          )
        }

        parent_offset -= 1;
      }
    }
  }
}