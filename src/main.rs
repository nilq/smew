extern crate colored;
extern crate backtrace;

mod smew;

use self::smew::source::*;
use self::smew::lexer::*;
use self::smew::parser::*;
use self::smew::interpreter::*;

fn main() {
  let test = r#"
a = 100

foo:
  "/res/sprite.png"

  b = a ++ 100

  baz:
    b
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
