mod compiler;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lea.pest"]
struct LeaParser;

fn main() {
    let a = "module Main; fn add(a: i32, b: i32) -> i32 {} fn main() { print('Hello, World!'); print(); }";

    match LeaParser::parse(Rule::source, a) {
        Err(e) => println!("{e}"),
        Ok(mut parsed) => {
            println!("{parsed:#?}");
            let mut pairs = parsed.next().unwrap().into_inner();
            let class = compiler::compile(&mut pairs).unwrap();
            std::fs::write("out.class", &class).unwrap();
        }
    }
}
