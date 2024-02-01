mod compiler;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lea.pest"]
struct LeaParser;

fn main() {
    let file = std::env::args().skip(1).next().unwrap_or("main.lea".to_string());
    println!("{:?}", file);
    let src = std::fs::read_to_string(file).unwrap();

    match LeaParser::parse(Rule::source, &src) {
        Err(e) => println!("{e}"),
        Ok(mut parsed) => {
            let mut pairs = parsed.next().unwrap().into_inner();
            let mut class = compiler::compile(&mut pairs).unwrap();
            std::fs::write(format!("{}.class", class.this_class), &class.serialize()).unwrap();
        }
    }
}
