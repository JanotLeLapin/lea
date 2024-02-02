#[derive(Debug)]
pub enum CompileErrorId {
    SymbolNotFound(String),
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

        println!("{fileame} --> {}:{}", self.start.0, self.start.1);
        println!("|");

        let line = source.split("\n").collect::<Vec<_>>()[self.start.0-1];
        let offset = line.char_indices().find_map(|(i, c)| if c == ' ' { None } else { Some(i) }).unwrap_or(0);

        let line = &line[offset..];

        println!("| {line}");

        let mut arrow = String::new();
        for _ in 0..self.start.1-1-offset { arrow.push(' '); }
        arrow.push('^');
        println!("| {arrow}");

        let msg = match &self.id {
            SymbolNotFound(symbol) => format!("= symbol not found: {}", symbol),
        };

        println!("{msg}");
    }
}

pub type Result<T> = std::result::Result<T, Vec<CompileError>>;
