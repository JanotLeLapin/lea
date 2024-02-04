use std::collections::HashMap;

use crate::{Rule, compiler::t::TypeId};
use super::t::{Type, Descriptor};

use bytes::{BufMut, BytesMut};
use pest::iterators::{Pair, Pairs};

pub struct MethodCompiler<'a> {
    cp: &'a mut super::constant_pool::ConstantPool,
    name: &'a str,
    descriptor: Descriptor,

    args: HashMap<&'a str, (Type, u8)>,
    vars: HashMap<&'a str, (Type, u8)>,
    pub errs: Vec<super::CompileError>,
    b: BytesMut,
}

impl<'a> MethodCompiler<'a> {
    pub fn new(cp: &'a mut super::constant_pool::ConstantPool, method: &Method<'a>) -> Self {
        Self {
            cp,
            name: method.name,
            descriptor: method.descriptor.clone(),
            args: method.args.clone(),
            vars: HashMap::new(),
            errs: vec![],
            b: BytesMut::new(),
        }
    }

    pub fn compile(&mut self, pair: Pair<'a, Rule>, class: &super::ClassFile<'a>) -> Vec<u8> {
        let pairs = pair.into_inner();

        let mut returned = false;
        for pair in pairs {
            match pair.as_rule() {
                Rule::returnStmt => {
                    match self.compile_value(pair.into_inner().next().unwrap(), class).unwrap().id {
                        TypeId::I8 | TypeId::I16 | TypeId::I32 | TypeId::I64 | TypeId::Char | TypeId::Bool => self.b.put_u8(172), // ireturn
                        _ => self.b.put_u8(176), // areturn
                    };
                    returned = true;
                },
                Rule::callExpr => self.compile_call_expr(pair, class),
                Rule::varDecl => self.compile_var_decl(pair, class),
                _ => {
                    println!("{pair:?}");
                },
            }
        }

        if !returned { self.b.put_u8(177); }

        let mut body = bytes::BytesMut::new();
        body.put_u16(16);
        body.put_u16(16);
        body.put_u32(self.b.len() as u32);
        body.put_slice(&self.b);
        body.put_u16(0);
        body.put_u16(0);

        let mut buf = bytes::BytesMut::new();
        buf.put_u16(self.cp.insert_utf8("Code".to_string()));
        buf.put_u32(body.len() as u32);
        buf.put_slice(&body);

        let mut res = bytes::BytesMut::new();
        res.put_u16(1 | 8);
        res.put_u16(self.cp.insert_utf8(self.name.to_string()));
        res.put_u16(self.cp.insert_utf8(self.descriptor.to_string()));

        res.put_u16(1);
        res.put_slice(&buf);

        res.to_vec()
    }

    fn compile_call_expr(&mut self, pair: Pair<'a, Rule>, class: &super::ClassFile<'a>) {
        let mut pairs = pair.into_inner();

        let ident = pairs.next().unwrap().as_str();

        match ident {
            "print" => {
                self.b.put_u8(178); // getstatic
                self.b.put_u16(self.cp.insert_ref(crate::compiler::constant_pool::Ref::Field, "java/lang/System".to_string(), "out".to_string(), "Ljava/io/PrintStream;".to_string()));
                let descriptor = self.compile_args(pairs, class);
                self.b.put_u8(182); // invokevirtual
                self.b.put_u16(self.cp.insert_ref(crate::compiler::constant_pool::Ref::Method, "java/io/PrintStream".to_string(), "println".to_string(), descriptor.to_string()));
            },
            f => {
                self.compile_args(pairs, class);
                self.b.put_u8(184); // invokestatic
                let method = class.methods.get(f).unwrap();
                self.b.put_u16(self.cp.insert_ref(crate::compiler::constant_pool::Ref::Method, class.this_class.clone(), f.to_string(), method.descriptor.to_string()));
            },
        }
    }

    fn compile_var_decl(&mut self, pair: Pair<'a, Rule>, class: &super::ClassFile<'a>) {
        let mut pairs = pair.into_inner();
        let ident = pairs.next().unwrap().as_str();
        let t = {
            let pair = pairs.peek().unwrap();
            match pair.as_rule() {
                Rule::primitive | Rule::object => {
                    let t_id = pairs.next().unwrap().as_str().parse().unwrap();
                    let is_array = match pairs.peek().unwrap().as_rule() {
                        Rule::array => {
                            pairs.next();
                            true
                        },
                        _ => false,
                    };

                    Some(Type::new(t_id, is_array))
                }
                _ => None,
            }
        };
        let v = pairs.next().unwrap();
        let t = t.unwrap_or(self.compile_value(v, class).unwrap());

        let store_idx = (self.args.len() + self.vars.len()) as u8;

        match t.id {
            TypeId::I8 | TypeId::I16 | TypeId::I32 | TypeId::Char | TypeId::Bool => self.b.put_u8(59 + store_idx as u8), // istore_n
            TypeId::Other(_) => self.b.put_u8(75 + store_idx as u8), // astore_n
            t => { unimplemented!("{t:?}") },
        };

        self.vars.insert(ident, (t, store_idx));
    }

    pub fn compile_args(&mut self, pairs: Pairs<'a, Rule>, class: &super::ClassFile<'a>) -> Descriptor {
        let mut descriptor = Descriptor::new(Vec::new(), Type::new(super::t::TypeId::Void, false));
        for arg in pairs {
            match self.compile_value(arg, class) {
                Ok(t) => descriptor.args.push(t),
                Err(e) => self.errs.push(e),
            }
        }

        descriptor
    }

    pub fn compile_value(&mut self, value: Pair<'a, Rule>, class: &super::ClassFile<'a>) -> Result<Type, super::CompileError> {
        Ok(match value.as_rule() {
            Rule::ident => {
                let arg_v = value.as_str();
                let (t, idx) =
                    if let Some(v) = self.args.get(arg_v) { v }
                    else if let Some(v) = self.vars.get(arg_v) { v }
                    else {
                        return Err(super::CompileError::new(super::CompileErrorId::SymbolNotFound(arg_v.to_string()), value.line_col()));
                    };
                match t.id {
                    TypeId::I8 | TypeId::I16 | TypeId::I32 | TypeId::Char | TypeId::Bool
                        => self.b.put_u8(26 + *idx), // iload_n
                    TypeId::Other(_) => self.b.put_u8(42 + *idx), // aload_n
                    _ => {},
                };

                t.clone()
            },
            Rule::callExpr => {
                let mut pairs = value.into_inner();
                let ident = pairs.next().unwrap();
                let method = class.methods.get(ident.as_str()).unwrap();

                self.b.put_u8(184); // invokestatic
                self.b.put_u16(self.cp.insert_ref(super::constant_pool::Ref::Method, class.this_class.clone(), ident.as_str().to_string(), method.descriptor.to_string()));

                method.descriptor.return_type.clone()
            },
            r => {
                match r {
                    Rule::numLit => {
                        self.b.put_u8(16); // bipush
                        self.b.put_u8(value.as_str().parse().unwrap());
                        Type::new(TypeId::I32, false)
                    },
                    Rule::strLit => {
                        self.b.put_u8(18); // ldc
                        self.b.put_u8(self.cp.insert_string(value.into_inner().next().unwrap().as_str().to_string()) as u8);
                        Type::new(TypeId::Other("String".to_string()), false)
                    },
                    Rule::charLit => {
                        self.b.put_u16(16); // bipush
                        self.b.put_u8(value.as_str().chars().skip(1).next().unwrap() as u32 as u8);
                        Type::new(TypeId::Char, false)
                    }
                    Rule::boolLit => {
                        self.b.put_u16(16); // bipush
                        self.b.put_u8(match value.as_str() {
                            "true" => 1,
                            _ => 0,
                        });
                        Type::new(TypeId::Bool, false)
                    },
                    _ => { unimplemented!("{r:?}") }
                }
            }
        })
    }
}

#[derive(Debug)]
pub struct Method<'a> {
    pub name: &'a str,
    pub descriptor: Descriptor,
    pub args: HashMap<&'a str, (Type, u8)>,
    pub code: pest::iterators::Pair<'a, Rule>,
}

impl<'a> Method<'a> {
    pub fn parse(pairs: &mut pest::iterators::Pairs<'a, Rule>) -> Self {
        let ident = pairs.next().unwrap().as_str();

        let mut params = vec![];
        let next = loop {
            let node = pairs.next().unwrap();
            if node.as_rule() != Rule::param { break node; }

            let mut pairs = node.into_inner();
            let ident = pairs.next().unwrap().as_str();
            let t = Type::new(pairs.next().unwrap().as_str().parse().unwrap(), pairs.next().is_some());

            params.push((ident, t));
        };

        let (ret_type, block) =
            if next.as_rule() == Rule::block { (Type::new(super::t::TypeId::Void, false), next) }
            else { (Type::new(next.as_str().parse().unwrap(), false), pairs.next().unwrap()) };

        let mut arg_map = HashMap::new();
        let mut arg_lst = vec![];
        for (ident, t) in params {
            arg_map.insert(ident, (t.clone(), arg_map.len() as u8));
            arg_lst.push(t);
        }
        
        let descriptor = Descriptor::new(arg_lst, ret_type);

        Self {
            name: ident,
            descriptor,
            args: arg_map,
            code: block,
        }
    }
}
