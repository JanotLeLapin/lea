use bytes::{BufMut, BytesMut};

use crate::Rule;

pub fn compile_code<'a>(pairs: &mut pest::iterators::Pairs<'a, Rule>, cp: &mut crate::compiler::constant_pool::ConstantPool) -> Vec<u8> {
    let mut code = BytesMut::new();
    code.put_u8(178); // getstatic
    code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Field, "java/lang/System".to_string(), "out".to_string(), "Ljava/io/PrintStream;".to_string()));
    code.put_u8(18); // ldc
    code.put_u8(cp.insert_string("Hello, World!".to_string()) as u8);
    code.put_u8(182); // invokevirtual
    code.put_u16(cp.insert_ref(crate::compiler::constant_pool::Ref::Method, "java/io/PrintStream".to_string(), "println".to_string(), "(Ljava/lang/String;)V".to_string()));
    code.put_u8(177); // return

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
