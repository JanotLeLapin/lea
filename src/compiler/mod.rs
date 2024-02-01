pub mod constant_pool;
pub mod method;

use crate::Rule;

use bytes::BufMut;

#[derive(Debug)]
pub struct Version {
    minor: u16,
    major: u16,
}

impl Version {
    pub fn new(minor: u16, major: u16) -> Self {
        Self { minor, major }
    }
}

#[derive(Debug)]
pub struct ClassFile {
    magic: u32,
    version: Version,
    constant_pool: constant_pool::ConstantPool,
    access_flags: u16,
    this_class: String,
    super_class: String,
    methods: Vec<Vec<u8>>,
}

impl ClassFile {
    pub fn new(magic: u32, version: Version, access_flags: u16, this_class: String, super_class: String) -> Self {
        Self {
            magic,
            version,
            constant_pool: constant_pool::ConstantPool::new(),
            access_flags,
            this_class,
            super_class,
            methods: Vec::new(),
        }
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        let mut body = bytes::BytesMut::new();
        body.put_u16(self.access_flags);
        body.put_u16(self.constant_pool.insert_class(self.this_class.clone()));
        body.put_u16(self.constant_pool.insert_class(self.super_class.clone()));

        body.put_u16(0);

        body.put_u16(0);

        body.put_u16(self.methods.len() as u16);
        for method in &self.methods {
            body.put_slice(&method);
        }

        body.put_u16(0);

        let mut buf = bytes::BytesMut::new();

        buf.put_u32(self.magic);
        buf.put_u16(self.version.minor);
        buf.put_u16(self.version.major);

        buf.put_slice(&self.constant_pool.serialize());
        buf.put_slice(&body);

        buf.to_vec()
    }
}

#[derive(Debug)]
pub enum CompileError {
    ExpectedModule,
}

pub fn parse_type<'a>(t: &'a str) -> &'a str {
    match t {
        "String" => "Ljava/lang/String;",
        "i32" => "I",
        t => t,
    }
}

pub fn compile<'a>(ast: &mut pest::iterators::Pairs<'a, Rule>) -> Result<Vec<u8>, CompileError> {
    let module = ast.next().unwrap();
    if module.as_rule() != Rule::module_statement { return Err(CompileError::ExpectedModule) }

    let mut class = ClassFile::new(
        0xCAFEBABE,
        Version::new(0, 52),
        1 | 32,
        module.into_inner().next().unwrap().as_str().to_string(),
        "java/lang/Object".to_string()
    );

    for node in ast {
        match node.as_rule() {
            Rule::function_declaration => {
                let method = self::method::compile_method(&mut node.into_inner(), &mut class.constant_pool);
                class.methods.push(method);
            },
            _ => {},
        }
    }

    let compiled = class.serialize();
    Ok(compiled)
}
