use colored::Colorize;

#[derive(Debug)]
pub enum CompileErrorId {
    SymbolNotFound(String),
    UnexpectedArgCount(u16),
    UnexpectedArgType(String, String),
}

#[derive(Debug)]
pub struct CompileError {
    id: CompileErrorId,
    start: (usize, usize),
}

impl CompileError {
    pub fn new(id: CompileErrorId, start: (usize, usize)) -> Self {
        Self { id, start }
    }

    pub fn print(&self, fileame: &str, source: &str) {
        use CompileErrorId::*;

        let bar = "|".blue();

        println!("{fileame} {} {}:{}", "-->".blue(), self.start.0, self.start.1);
        println!("{bar}");

        let line = source.split("\n").collect::<Vec<_>>()[self.start.0-1];
        let offset = line.char_indices().find_map(|(i, c)| if c == ' ' { None } else { Some(i) }).unwrap_or(0);

        let line = &line[offset..];

        println!("{bar} {line}");

        let mut arrow = String::new();
        for _ in 0..self.start.1-1-offset { arrow.push(' '); }
        arrow.push('^');
        println!("{bar} {}", arrow.red());

        let msg = match &self.id {
            SymbolNotFound(symbol) => format!("symbol not found: {}", symbol),
            UnexpectedArgCount(expected) => format!("unexpected argument: expected {} arguments", expected),
            UnexpectedArgType(expected, got) => format!("unexpected argument: expected a {}, got a {}", expected, got),
        };

        println!("{} {}: {msg}", "=".blue(), "error".red());
    }
}

pub type Result<T> = std::result::Result<T, Vec<CompileError>>;
