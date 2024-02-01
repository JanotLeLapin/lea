use std::collections::HashMap;

use crate::Rule;

use bytes::BufMut;

#[derive(Debug)]
pub struct Method<'a> {
    pub name: &'a str,
    descriptor: String,
    args: HashMap<String, (String, u16)>,
    code: pest::iterators::Pair<'a, Rule>,
}

impl<'a> Method<'a> {
    pub fn parse(pairs: &mut pest::iterators::Pairs<'a, Rule>) -> Self {
        let ident = pairs.next().unwrap().as_str();

        let mut params = vec![];
        let next = loop {
            let node = pairs.next().unwrap();
            if node.as_rule() != Rule::parameter { break node; }

            let mut pairs = node.into_inner();
            let ident = pairs.next().unwrap().as_str();
            let t = crate::compiler::parse_type(pairs.next().unwrap().as_str());
            let is_array = pairs.next().is_some();

            params.push((ident, if is_array { format!("[{}", t) } else { t.to_string() }));
        };

        let (ret_type, block) =
            if next.as_rule() == Rule::block { ("V", next) }
            else {
                (crate::compiler::parse_type(next.as_str()), pairs.next().unwrap())
            };

        let mut map = HashMap::new();
        for (ident, t) in &params {
            map.insert(ident.to_string(), (t.to_string(), map.len() as u16));
        }

        Self {
            name: ident,
            descriptor: format!("({}){}", params.into_iter().map(|(_, b)| b).collect::<Vec<_>>().join(""), ret_type),
            args: map,
            code: block,
        }
    }

    pub fn compile_code(&self, methods: &super::MethodMap, cp: &mut crate::compiler::constant_pool::ConstantPool) -> Vec<u8> {
        let pairs = self.code.clone().into_inner();

        let mut code = bytes::BytesMut::new();

        let mut returned = false;
        for pair in pairs {
            match pair.as_rule() {
                Rule::return_statement => {
                    code.put_u8(18); // ldc
                    code.put_u8(cp.insert_string(pair.into_inner().as_str().to_string()) as u8);
                    code.put_u8(176); // areturn
                    returned = true;
                },
                Rule::call_expression => {
                    let mut pairs = pair.into_inner();

                    match pairs.next().unwrap().as_str() {
                        "print" => {
                            code.put_u8(178); // getstatic
                            code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Field, "java/lang/System".to_string(), "out".to_string(), "Ljava/io/PrintStream;".to_string()));
                            for arg in pairs {
                                match arg.as_rule() {
                                    Rule::ident => {
                                        let (_, idx) = self.args.get(arg.as_str()).unwrap();
                                        code.put_u8(42 + *idx as u8); // aload_n
                                    },
                                    _ => {
                                        code.put_u8(18); // ldc
                                        code.put_u8(cp.insert_string(arg.as_str().to_string()) as u8);
                                    }
                                }
                            }
                            code.put_u8(182); // invokevirtual
                            code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Method, "java/io/PrintStream".to_string(), "println".to_string(), "(Ljava/lang/String;)V".to_string()));
                        },
                        f => {
                            let method = methods.get(f).unwrap();
                            for arg in pairs {
                                match arg.as_rule() {
                                    Rule::ident => {
                                        code.put_u8(42); // aload_0
                                    },
                                    _ => {
                                        code.put_u8(18); // ldc
                                        code.put_u8(cp.insert_string(arg.as_str().to_string()) as u8);
                                    }
                                }
                            }
                            code.put_u8(184); // invokestatic
                            code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Method, "Main".to_string(), f.to_string(), method.descriptor.clone()));
                        },
                    }
                }
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

        buf.to_vec()
    }

    pub fn compile(&self, cp: &mut crate::compiler::constant_pool::ConstantPool, compiled_code: Vec<u8>) -> Vec<u8> {
        let mut buf = bytes::BytesMut::new();
        buf.put_u16(1 | 8);
        buf.put_u16(cp.insert_utf8(self.name.to_string()));
        buf.put_u16(cp.insert_utf8(self.descriptor.clone()));

        buf.put_u16(1);
        buf.put_slice(&compiled_code);
        buf.to_vec()
    }
}
