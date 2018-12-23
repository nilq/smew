extern crate colored;

mod smew;

use self::smew::source::*;
use self::smew::lexer::*;

fn main() {
  let test = r#"
player: 
  looks:
    sprite:
      "/res/sprites/player.png"

      scale-x: 10
      scale-y: 10

  when-awake:
    print "hello world"
    move-to 100, 200 - 100

  when-press-space:
    print "ouch holy fuck"
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

  println!("{:#?}", tokens);
}
