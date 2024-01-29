use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lea.pest"]
struct LeaParser;

fn main() {
    let a = "fn add(a: i32, b: i32) -> i32 {}";

    match LeaParser::parse(Rule::source, a) {
        Err(e) => println!("{e}"),
        Ok(parsed) => println!("{parsed:#?}")
    }
}
