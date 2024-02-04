pub mod constant_pool;
pub mod method;
pub mod structure;
pub mod t;

pub mod error;
pub use error::{CompileErrorId, CompileError, Result};

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

    pub fn compile(&mut self, ast: &mut pest::iterators::Pairs<'a, Rule>) -> Result<Vec<u8>> {
        let mut errs = vec![];

        let mut cp = constant_pool::ConstantPool::new();

        for node in ast {
            match node.as_rule() {
                Rule::functionDecl => {
                    let method = self::method::Method::parse(&mut node.into_inner());
                    self.methods.insert(method.name.to_string(), method);
                },
                Rule::structDecl => {
                    let structure = self::structure::Structure::parse(&mut node.into_inner());
                    self.structures.insert(structure.name.to_string(), structure);
                }
                _ => {
                },
            }
        }

        let mut body = bytes::BytesMut::new();
        body.put_u16(self.access_flags);
        body.put_u16(cp.insert_class(self.this_class.to_string()));
        body.put_u16(cp.insert_class(self.super_class.to_string()));

        body.put_u16(0);

        body.put_u16(0);

        body.put_u16(self.methods.len() as u16);
        for (_, method) in &self.methods {
            let mut ctx = method::MethodCompiler::new(&mut cp, method);
            body.put_slice(&ctx.compile(method.code.clone(), &self));
            errs.append(&mut ctx.errs);
        }

        body.put_u16(0);

        let mut buf = bytes::BytesMut::new();

        buf.put_u32(self.magic);
        buf.put_u16(self.version.minor);
        buf.put_u16(self.version.major);

        buf.put_slice(&cp.serialize());
        buf.put_slice(&body);

        if errs.len() == 0 { Ok(buf.to_vec()) }
        else { Err(errs) }
    }
}
