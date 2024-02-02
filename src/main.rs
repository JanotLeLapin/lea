mod compiler;

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lea.pest"]
struct LeaParser;

fn main() {
    let file = std::env::args().skip(1).next().unwrap_or("main.lea".to_string());
    println!("{:?}", file);
    let src = std::fs::read_to_string(&file).unwrap();

    match LeaParser::parse(Rule::source, &src) {
        Err(e) => println!("{e}"),
        Ok(mut parsed) => {
            let mut pairs = parsed.next().unwrap().into_inner();
            let (class, data) = match compiler::compile(&mut pairs).and_then(|mut class| class.serialize().map(|data| (class, data))) {
                Ok(v) => v,
                Err(e) => {
                    e.print(&file, &src);
                    return;
                },
            };
            std::fs::write(format!("{}.class", class.this_class), &data).unwrap();
        }
    }
}
