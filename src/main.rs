extern crate colored;
extern crate backtrace;

mod smew;

use self::smew::source::*;
use self::smew::lexer::*;
use self::smew::parser::*;
use self::smew::interpreter::*;

fn main() {
  let test = r#"
rectangle:
  width:  200
  height: 100

vector2:
  x: 100
  y: 100

area: rectangle.width * rectangle.height
  "#;

  let source = Source::from("<main>", test.lines().map(|x| x.into()).collect::<Vec<String>>());
  let lexer  = Lexer::default(test.chars().collect(), &source);

  let mut tokens = Vec::new();

  for token_result in lexer {
    if let Ok(token) = token_result {
      tokens.push(token)
    } else {
      return
    }
  }

  let mut parser  = Parser::new(tokens, &source);

  match parser.parse() {
    Ok(ref ast) => {

      let mut interpreter = Interpreter::new(&source);

      match interpreter.evaluate(ast) {
        scope => println!("{:#?}", scope),
      }
    },

    _ => ()
  }
}
