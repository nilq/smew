extern crate colored;
extern crate backtrace;
extern crate rustyline;

mod smew;

use self::smew::source::*;
use self::smew::lexer::*;
use self::smew::parser::*;
use self::smew::interpreter::*;

use std::collections::HashMap;
use rustyline::Editor;

fn print(args: &Vec<Object>) -> Object {
  for arg in args {
    if let Some(ref string) = arg.to_str_object() {
      print!("{}", string)
    }
  }

  println!();

  Object::Nil
}

fn input(args: &Vec<Object>) -> Object {
  let mut rl = Editor::<()>::new();

  let mut result = String::new();

  let readline = rl.readline("");

  match readline {
    Ok(line) => {
      result = line;
    },

    _ => (),
  }

  Object::Str(result)
}

fn color(args: &Vec<Object>) -> Object {
  use colored::Colorize;

  let color = format!("{}", args[0].to_str_object().unwrap());
  let text  = format!("{}", args[1].to_str_object().unwrap());

  Object::Str(format!("{}", text.color(color)))
}

fn main() {
  let test = r#"
rectangle:
  width:  100
  height: 100

area = rectangle.width * rectangle.height

status1 = color("blue", "my fav area is") ++ color("red", "== ") ++ area

print()
print(status1 ++ "\n")

brain:
  thought: "i actually really like magenta"

print(color("magenta", brain.thought))

print("what's your thought on sushi?")
your_thought_on_sushi = color("green", input())

print()

print("your though on sushi:")
print(your_thought_on_sushi)
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
      let mut foreign = HashMap::new();

      foreign.insert("print".to_string(), print as ForeignFunction);
      foreign.insert("color".to_string(), color as ForeignFunction);
      foreign.insert("input".to_string(), input as ForeignFunction);

      let mut interpreter = Interpreter::new(&source, foreign);

      match interpreter.evaluate(ast) {
        _ => (),
      }
    },

    _ => ()
  }
}
