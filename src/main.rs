mod compiler;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lea.pest"]
struct LeaParser;

fn main() {
    let src = std::fs::read_to_string("./main.lea").unwrap();

    match LeaParser::parse(Rule::source, &src) {
        Err(e) => println!("{e}"),
        Ok(mut parsed) => {
            let mut pairs = parsed.next().unwrap().into_inner();
            let class = compiler::compile(&mut pairs).unwrap();
            std::fs::write("Main.class", &class).unwrap();
        }
    }
}
