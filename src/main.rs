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
            let this = pairs.next().unwrap().into_inner().as_str();

            let mut class = compiler::ClassFile::new(
                0xCAFEBABE,
                compiler::Version::new(0, 52),
                1 | 32,
                this.to_string(), "java/lang/Object".to_string(),
            );
            match class.compile(&mut pairs) {
                Ok(data) => std::fs::write(format!("{}.class", this), data).unwrap(),
                Err(errs) => for err in errs {
                    err.print(&file, &src);
                }
            };
        }
    }
}
