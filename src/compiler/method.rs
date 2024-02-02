use std::collections::HashMap;

use crate::{Rule, compiler::t::TypeId};
use super::t::{Type, Descriptor};

use bytes::BufMut;

#[derive(Debug)]
pub struct Method<'a> {
    pub name: &'a str,
    descriptor: Descriptor,
    args: HashMap<String, (Type, u8)>,
    code: pest::iterators::Pair<'a, Rule>,
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
            arg_map.insert(ident.to_string(), (t.clone(), arg_map.len() as u8));
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

    pub fn compile_code(&self, class: &crate::compiler::ClassFile<'a>, cp: &mut crate::compiler::constant_pool::ConstantPool) -> super::Result<Vec<u8>> {
        let mut vars = HashMap::new();

        let pairs = self.code.clone().into_inner();

        let mut code = bytes::BytesMut::new();

        let mut returned = false;
        for pair in pairs {
            match pair.as_rule() {
                Rule::returnStmt => {
                    code.put_u8(18); // ldc
                    code.put_u8(cp.insert_string(pair.into_inner().as_str().to_string()) as u8);
                    code.put_u8(176); // areturn
                    returned = true;
                },
                Rule::callExpr => {
                    let mut pairs = pair.into_inner();

                    let ident = pairs.next().unwrap().as_str();

                    match ident {
                        "print" => {
                            code.put_u8(178); // getstatic
                            code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Field, "java/lang/System".to_string(), "out".to_string(), "Ljava/io/PrintStream;".to_string()));
                        },
                        _ => {},
                    }

                    let mut descriptor = Descriptor::new(Vec::new(), Type::new(super::t::TypeId::Void, false));
                    for arg in pairs {
                        match arg.as_rule() {
                            Rule::ident => {
                                let arg_v = arg.as_str();
                                let (t, idx) = (
                                    if let Some(v) = self.args.get(arg_v) { Ok(v) }
                                    else if let Some(v) = vars.get(arg_v) { Ok(v) }
                                    else { Err(super::CompileError::new(super::CompileErrorId::SymbolNotFound(arg_v.to_string()), arg.line_col())) }
                                )?;
                                descriptor.args.push(t.clone());
                                match t.id {
                                    TypeId::I8 | TypeId::I16 | TypeId::I32 | TypeId::Char | TypeId::Bool
                                        => code.put_u8(26 + *idx), // iload_n
                                    TypeId::Other(_) => code.put_u8(42 + *idx), // aload_n
                                    _ => {},
                                }
                            },
                            _ => {
                                code.put_u8(18); // ldc
                                code.put_u8(cp.insert_string(arg.into_inner().next().unwrap().as_str().to_string()) as u8);
                            }
                        }
                    }

                    match ident {
                        "print" => {
                            code.put_u8(182); // invokevirtual
                            code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Method, "java/io/PrintStream".to_string(), "println".to_string(), descriptor.to_string()));
                        },
                        f => {
                            code.put_u8(184); // invokestatic
                            let method = class.methods.get(f).unwrap();
                            code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Method, class.this_class.clone(), f.to_string(), method.descriptor.to_string()));
                        },
                    }
                },
                Rule::varDecl => {
                    let mut pairs = pair.into_inner();
                    let ident = pairs.next().unwrap().as_str();
                    let t = Type::new(pairs.next().unwrap().as_str().parse().unwrap(), false);
                    let v = pairs.next().unwrap();

                    let store_idx = (self.args.len() + vars.len()) as u8;

                    match t.id {
                        TypeId::I8 | TypeId::I16 | TypeId::I32 => {
                            code.put_u8(16); // bipush
                            code.put_u8(v.as_str().parse().unwrap());
                            code.put_u8(59 + store_idx as u8); // istore_n
                        },
                        TypeId::Char => {
                            code.put_u16(16); // bipush
                            code.put_u8(v.as_str().chars().skip(1).next().unwrap() as u32 as u8);
                            code.put_u8(59 + store_idx as u8) // istore_n
                        },
                        TypeId::Bool => {
                            code.put_u16(16); // bipush
                            code.put_u8(match v.as_str() {
                                "true" => 1,
                                _ => 0,
                            });
                            code.put_u8(59 + store_idx as u8) // istore_n
                        }
                        TypeId::Other(_) => {
                            code.put_u8(18); // ldc
                            code.put_u8(cp.insert_string(v.into_inner().as_str().to_string()) as u8);
                            code.put_u8(75 + store_idx as u8); // astore_n
                        },
                        _ => {},
                    }

                    vars.insert(ident, (t, store_idx));
                },
                _ => {
                    println!("{pair:?}");
                },
            }
        }

        if !returned { code.put_u8(177); }

        let mut body = bytes::BytesMut::new();
        body.put_u16(16);
        body.put_u16(16);
        body.put_u32(code.len() as u32);
        body.put_slice(&code);
        body.put_u16(0);
        body.put_u16(0);

        let mut buf = bytes::BytesMut::new();
        buf.put_u16(cp.insert_utf8("Code".to_string()));
        buf.put_u32(body.len() as u32);
        buf.put_slice(&body);

        Ok(buf.to_vec())
    }

    pub fn compile(&self, cp: &mut crate::compiler::constant_pool::ConstantPool, compiled_code: Vec<u8>) -> Vec<u8> {
        let mut buf = bytes::BytesMut::new();
        buf.put_u16(1 | 8);
        buf.put_u16(cp.insert_utf8(self.name.to_string()));
        buf.put_u16(cp.insert_utf8(self.descriptor.to_string()));

        buf.put_u16(1);
        buf.put_slice(&compiled_code);
        buf.to_vec()
    }
}
