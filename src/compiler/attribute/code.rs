use bytes::{BufMut, BytesMut};

use crate::Rule;

pub fn compile_code<'a>(code: pest::iterators::Pair<'a, Rule>, methods: &crate::compiler::MethodMap, cp: &mut crate::compiler::constant_pool::ConstantPool) -> Vec<u8> {
    let pairs = code.into_inner();

    let mut code = BytesMut::new();

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
                        code.put_u8(18); // ldc
                        code.put_u8(cp.insert_string(pairs.next().unwrap().as_str().to_string()) as u8);
                        code.put_u8(182); // invokevirtual
                        code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Method, "java/io/PrintStream".to_string(), "println".to_string(), "(Ljava/lang/String;)V".to_string()));
                    },
                    f => {
                        code.put_u8(184); // invokestatic
                        let method = methods.get(f).unwrap();
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

    let mut body = BytesMut::new();
    body.put_u16(16);
    body.put_u16(16);
    body.put_u32(code.len() as u32);
    body.put_slice(&code);
    body.put_u16(0);
    body.put_u16(0);

    let mut buf = BytesMut::new();
    buf.put_u16(cp.insert_utf8("Code".to_string()));
    buf.put_u32(body.len() as u32);
    buf.put_slice(&body);

    buf.to_vec()
}
