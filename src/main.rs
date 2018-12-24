extern crate colored;
extern crate backtrace;

mod smew;

use self::smew::source::*;
use self::smew::lexer::*;
use self::smew::parser::*;

fn main() {
  let test = r#"
box:
  size:
    width:
      32
    
    height:
      32

player -> box:
  area:
    size.width * size.height

enemy:
  hey
  
  head:
    eyes
    a-nose
  
  bye
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
