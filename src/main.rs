use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lea.pest"]
struct LeaParser;

fn main() {
    let a = "fn main() { print('Hello, World!'); }";

    match LeaParser::parse(Rule::source, a) {
        Err(e) => println!("{e}"),
        Ok(parsed) => println!("{parsed:#?}")
    }
}
