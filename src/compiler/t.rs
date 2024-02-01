#[derive(Debug, Clone)]
pub enum TypeId {
    I8, I16, I32, I64,
    Char, Bool,
    Void,
    Other(String),
}

impl std::str::FromStr for TypeId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use TypeId::*;

        Ok(match s {
            "i8" => I8,
            "i16" => I16,
            "i32" => I32,
            "i64" => I64,
            "char" => Char,
            "bool" => Bool,
            s => Other(s.to_string()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Type {
    pub id: TypeId,
    array: bool,
}

impl Type {
    pub fn new(id: TypeId, array: bool) -> Self {
        Self { id, array }
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        use TypeId::*;

        let mut buf = String::new();
        if self.array { buf.push('[') }
        buf.push_str(match &self.id {
            I8 => "B",
            I16 => "S",
            I32 => "I",
            I64 => "J",
            Char => "C",
            Bool => "Z",
            Void => "V",
            Other(s) => match s.as_str() {
                "Object" => "Ljava/lang/Object;",
                "String" => "Ljava/lang/String;",
                s => s,
            },
        });
        buf
    }
}

#[derive(Debug)]
pub struct Descriptor {
    pub args: Vec<Type>,
    return_type: Type,
}

impl Descriptor {
    pub fn new(args: Vec<Type>, return_type: Type) -> Self {
        Self { args, return_type }
    }
}

impl ToString for Descriptor {
    fn to_string(&self) -> String {
        let mut buf = String::new();
        buf.push('(');
        for arg in &self.args {
            buf.push_str(&arg.to_string());
        }
        buf.push(')');
        buf.push_str(&self.return_type.to_string());
        buf
    }
}
