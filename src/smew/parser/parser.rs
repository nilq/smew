use super::*;
use super::super::error::Response::Wrong;

use std::rc::Rc;

pub struct Parser<'p> {
  index:  usize,
  tokens: Vec<Token>,
  source: &'p Source,
  indent: usize,
}

impl<'p> Parser<'p> {
  pub fn new(tokens: Vec<Token>, source: &'p Source) -> Self {
    Parser {
      tokens,
      source,
      index:  0,
      indent: 0,
    }
  }



  pub fn parse(&mut self) -> Result<Vec<Statement>, ()> {
    let mut ast = Vec::new();

    while self.remaining() > 0 {
      ast.push(self.parse_statement()?)
    }

    Ok(ast)
  }



  fn parse_statement(&mut self) -> Result<Statement, ()> {
    use self::TokenType::*;

    while self.current_type() == EOL && self.remaining() != 0 {
      self.next()?
    }

    let position = self.current_position();

    let statement = match self.current_type() {
      Identifier => {
        let backup_index = self.index;
        let name = self.eat()?;

        if self.current_lexeme() == ":" {
          self.next()?;
          self.next_newline()?;

          let body = self.parse_body()?;

          Statement::new(
            StatementNode::Record(
              name,
              body,
            ),
            position
          )
        } else {
          self.index = backup_index;

          let expression = self.parse_expression()?;
          let position   = expression.pos.clone();

          Statement::new(
            StatementNode::Expression(expression),
            position,
          )
        }
      },
      
      _ => {
        let expression = self.parse_expression()?;
        let position   = expression.pos.clone();

        if self.current_lexeme() == "=" {
          self.next()?;

          Statement::new(
            StatementNode::Assignment(expression, self.parse_expression()?),
            position
          )
        } else {
          Statement::new(
            StatementNode::Expression(expression),
            position,
          )
        }
      },
    };

    self.new_line()?;

    Ok(statement)
  }



  fn parse_body(&mut self) -> Result<Vec<Statement>, ()> {
    self.indent = self.get_indent();

    let mut stack = Vec::new();

    while !self.is_dedent() && self.remaining() > 1 {
      let statement = self.parse_statement()?;

      stack.push(statement)
    }

    println!("{} > {}", self.indent, self.get_indent());

    println!("{:#?}", stack);

    Ok(stack)
  }



  fn parse_expression(&mut self) -> Result<Expression, ()> {
    let atom = self.parse_atom()?;

    if self.current_type() == TokenType::Operator {
      self.parse_binary(atom)
    } else {
      Ok(atom)
    }
  }



  fn parse_atom(&mut self) -> Result<Expression, ()> {
    use self::TokenType::*;

    if self.remaining() == 0 {
      Ok(
        Expression::new(
          ExpressionNode::EOF,
          self.current_position()
        )
      )
    } else {
      let token_type = self.current_type().clone();
      let position   = self.current_position();

      let expression = match token_type {
        Number => Expression::new(
          ExpressionNode::Number(self.eat()?.parse::<f64>().unwrap()),
          position
        ),

        Str => Expression::new(
          ExpressionNode::Str(self.eat()?),
          position
        ),

        Bool => Expression::new(
          ExpressionNode::Bool(self.eat()? == "true"),
          position
        ),

        Identifier => Expression::new(
          ExpressionNode::Identifier(self.eat()?),
          position
        ),

        Operator => match self.current_lexeme().as_str() {
          "-" => {
            self.next()?;

            Expression::new(
              ExpressionNode::Neg(
                Rc::new(self.parse_expression()?)
              ),

              self.span_from(position)
            )
          },

          "not" => {
            self.next()?;

            Expression::new(
              ExpressionNode::Not(
                Rc::new(self.parse_expression()?)
              ),

              self.span_from(position)
            )
          },

          ref symbol => return Err(
            response!(
              Wrong(format!("unexpected operator `{}`", symbol)),
              self.source.file,
              self.current_position()
            )
          )
        },

        ref token_type => return Err(
          response!(
            Wrong(format!("unexpected token `{}`", token_type)),
            self.source.file,
            self.current_position()
          )
        )
      };

      if self.remaining() > 0 {
        self.parse_postfix(expression)
      } else {
        Ok(expression)
      }
    }
  }

  fn parse_postfix(&mut self, expression: Expression) -> Result<Expression, ()> {
    if self.remaining() == 0 {
      return Ok(expression)
    }

    match self.current_type() {
      _ => {
        Ok(expression)
      },

      _ => Ok(expression)
    }
  }



  fn parse_binary(&mut self, left: Expression) -> Result<Expression, ()> {
    let left_position = left.pos.clone();

    let mut expression_stack = vec!(left);
    let mut operator_stack   = vec!(Operator::from_str(&self.eat()?).unwrap());

    expression_stack.push(self.parse_atom()?);

    while operator_stack.len() > 0 {
      while self.current_type() == TokenType::Operator {
        let position               = self.current_position();
        let (operator, precedence) = Operator::from_str(&self.eat()?).unwrap();

        if precedence < operator_stack.last().unwrap().1 {
          let right = expression_stack.pop().unwrap();
          let left  = expression_stack.pop().unwrap();

          expression_stack.push(
            Expression::new(
              ExpressionNode::Binary(Rc::new(left), operator_stack.pop().unwrap().0, Rc::new(right)),
              self.current_position(),
            )
          );

          if self.remaining() > 0 {
            expression_stack.push(self.parse_atom()?);
            operator_stack.push((operator, precedence))
          } else {
            return Err(
              response!(
                Wrong("reached EOF in operation"),
                self.source.file,
                position
              )
            )
          }
        } else {
          expression_stack.push(self.parse_atom()?);
          operator_stack.push((operator, precedence))
        }
      }

      let right = expression_stack.pop().unwrap();
      let left  = expression_stack.pop().unwrap();

      expression_stack.push(
        Expression::new(
          ExpressionNode::Binary(Rc::new(left), operator_stack.pop().unwrap().0, Rc::new(right)),
          self.current_position(),
        )
      );
    }

    let expression = expression_stack.pop().unwrap();

    Ok(
      Expression::new(
        expression.node,
        self.span_from(left_position)
      )
    )
  }



  fn new_line(&mut self) -> Result<(), ()> {
    if self.remaining() > 0 {
      match self.current_lexeme().as_str() {
        "\n" => self.next(),
        _    => Err(
          response!(
            Wrong(format!("expected new line found: `{}`", self.current_lexeme())),
            self.source.file,
            self.current_position()
          )
        )
      }
    } else {
      Ok(())
    }
  }



  fn next_newline(&mut self) -> Result<(), ()> {
    while self.current_lexeme() == "\n" && self.remaining() > 0 {
      self.next()?
    }

    Ok(())
  }



  fn get_indent(&self) -> usize {
    self.current().slice.0
  }

  fn is_dedent(&self) -> bool {
    self.get_indent() < self.indent
  }



  fn next(&mut self) -> Result<(), ()> {
    if self.index <= self.tokens.len() {
      self.index += 1;
      Ok(())
    } else {
      Err(
        response!(
          Wrong("moving outside token stack"),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn remaining(&self) -> usize {
    self.tokens.len().saturating_sub(self.index)
  }

  fn current_position(&self) -> Pos {
    let current = self.current();

    Pos(
      current.line.clone(),
      current.slice
    )
  }

  fn span_from(&self, left_position: Pos) -> Pos {
    let Pos(ref line, ref slice) = left_position;
    let Pos(_, ref slice2)       = self.current_position();

    Pos(line.clone(), (slice.0, if slice2.1 < line.1.len() { slice2.1 } else { line.1.len() } ))
  }

  fn current(&self) -> Token {
    if self.index > self.tokens.len() - 1 {
      self.tokens[self.tokens.len() - 1].clone()
    } else {
      self.tokens[self.index].clone()
    }
  }

  fn eat(&mut self) -> Result<String, ()> {
    let lexeme = self.current().lexeme;
    self.next()?;

    Ok(lexeme)
  }

  fn eat_lexeme(&mut self, lexeme: &str) -> Result<String, ()> {
    if self.current_lexeme() == lexeme {
      let lexeme = self.current().lexeme;
      self.next()?;

      Ok(lexeme)
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", lexeme, self.current_lexeme())),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn eat_type(&mut self, token_type: &TokenType) -> Result<String, ()> {
    if self.current_type() == *token_type {
      let lexeme = self.current().lexeme.clone();
      self.next()?;

      Ok(lexeme)
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file,
          self.current_position()
        )
      )
    }
  }

  fn current_lexeme(&self) -> String {
    self.current().lexeme.clone()
  }

  fn current_type(&self) -> TokenType {
    self.current().token_type
  }

  fn expect_type(&self, token_type: TokenType) -> Result<(), ()> {
    if self.current_type() == token_type {
      Ok(())
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", token_type, self.current_type())),
          self.source.file
        )
      )
    }
  }

  fn expect_lexeme(&self, lexeme: &str) -> Result<(), ()> {
    if self.current_lexeme() == lexeme {
      Ok(())
    } else {
      Err(
        response!(
          Wrong(format!("expected `{}`, found `{}`", lexeme, self.current_lexeme())),
          self.source.file
        )
      )
    }
  }
}