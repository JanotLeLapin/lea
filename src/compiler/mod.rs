pub mod constant_pool;
pub mod method;
pub mod structure;
pub mod t;

use std::collections::HashMap;

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
pub struct ClassFile<'a> {
    magic: u32,
    version: Version,
    access_flags: u16,
    pub this_class: String,
    super_class: String,
    methods: HashMap<String, method::Method<'a>>,
    structures: HashMap<String, structure::Structure<'a>>,
}

impl<'a> ClassFile<'a> {
    pub fn new(magic: u32, version: Version, access_flags: u16, this_class: String, super_class: String) -> Self {
        Self {
            magic,
            version,
            access_flags,
            this_class,
            super_class,
            methods: HashMap::new(),
            structures: HashMap::new(),
        }
    }

    pub fn serialize(&mut self) -> Vec<u8> {
        let mut cp = constant_pool::ConstantPool::new();

        let mut body = bytes::BytesMut::new();
        body.put_u16(self.access_flags);
        body.put_u16(cp.insert_class(self.this_class.clone()));
        body.put_u16(cp.insert_class(self.super_class.clone()));

        body.put_u16(0);

        body.put_u16(0);

        body.put_u16(self.methods.len() as u16);
        for (_, method) in &self.methods {
            let compiled_code = method.compile_code(&self, &mut cp);
            body.put_slice(&method.compile(&mut cp, compiled_code));
        }

        body.put_u16(0);

        let mut buf = bytes::BytesMut::new();

        buf.put_u32(self.magic);
        buf.put_u16(self.version.minor);
        buf.put_u16(self.version.major);

        buf.put_slice(&cp.serialize());
        buf.put_slice(&body);

        buf.to_vec()
    }
}

#[derive(Debug)]
pub enum CompileError {
    ExpectedModule,
}

pub fn compile<'a>(ast: &mut pest::iterators::Pairs<'a, Rule>) -> Result<ClassFile<'a>, CompileError> {
    let module = ast.next().unwrap();
    if module.as_rule() != Rule::module { return Err(CompileError::ExpectedModule) }

    let mut class = ClassFile::new(
        0xCAFEBABE,
        Version::new(0, 52),
        1 | 32,
        module.into_inner().next().unwrap().as_str().to_string(),
        "java/lang/Object".to_string()
    );

    for node in ast {
        match node.as_rule() {
            Rule::functionDecl => {
                let method = self::method::Method::parse(&mut node.into_inner());
                class.methods.insert(method.name.to_string(), method);
            },
            Rule::structDecl => {
                let structure = self::structure::Structure::parse(&mut node.into_inner());
                class.structures.insert(structure.name.to_string(), structure);
            }
            _ => {
            },
        }
    }

    Ok(class)
}
