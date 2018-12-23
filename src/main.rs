extern crate colored;

mod smew;

use self::smew::source::*;
use self::smew::lexer::*;
use self::smew::parser::*;

fn main() {
  let test = r#"
# Literals

hello-world - -10
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
      println!("{:#?}", ast)
    },

    _ => ()
  }
}
