pub mod constant_pool;

use crate::{Rule, compiler::constant_pool::Ref};

use bytes::BufMut;

pub mod access {
    pub const PUBLIC: u16 = 0x0001;
    pub const SUPER: u16 = 0x0020;
}

#[derive(Debug)]
pub enum CompileError {
    ExpectedModule,
}

pub fn compile<'a>(ast: &mut pest::iterators::Pairs<'a, Rule>) -> Result<Vec<u8>, CompileError> {
    use CompileError::*;

    let module = match ast.next() {
        Some(pair) => match pair.as_rule() {
            Rule::module_statement => Some(pair.into_inner().as_str()),
            _ => None
        },
        _ => None,
    }.ok_or(ExpectedModule)?;

    let mut cp = crate::compiler::constant_pool::ConstantPool::new();
    cp.insert_class(module);
    cp.insert_class("java/lang/Object");

    let mut body = bytes::BytesMut::new();
    body.put_u16(access::PUBLIC | access::SUPER);
    body.put_u16(2);
    body.put_u16(4);
    body.put_u16(0);
    body.put_u16(0);
    body.put_u16(1);
    {
        // meta
        body.put_u16(9);
        let idx = cp.insert_utf8("main"); body.put_u16(idx);
        let idx = cp.insert_utf8("([Ljava/lang/String;)V"); body.put_u16(idx);

        // attributes
        body.put_u16(1);
        let idx = cp.insert_utf8("Code"); body.put_u16(idx);
        let attribute = &{
            let mut buf = bytes::BytesMut::new();
            buf.put_u16(2);
            buf.put_u16(1);

            let code = &{
                let mut buf = bytes::BytesMut::new();
                buf.put_u8(0xb2);
                let idx = cp.insert_ref(Ref::Field, "java/lang/System", "Ljava/io/PrintStream;", "out"); buf.put_u16(idx);
                buf.put_u8(0x12);
                let idx = cp.insert_string("Hello, World"); buf.put_u8(idx as u8);
                buf.put_u8(0xb6);
                let idx = cp.insert_ref(Ref::Method, "java/io/PrintStream", "(Ljava/lang/String;)V", "println"); buf.put_u16(idx);
                buf.put_u8(0xb1);

                buf
            }[..];
            buf.put_u32(code.len() as u32);
            buf.put(code);

            buf.put_u16(0);
            buf.put_u16(0);

            buf
        }[..];
        body.put_u32(attribute.len() as u32);
        body.put(attribute);
    }
    body.put_u16(0);

    let mut res = bytes::BytesMut::new();
    res.put_u32(0xCAFEBABE);
    res.put_u16(0);
    res.put_u16(52);

    res.put(&cp.serialize()[..]);
    res.put(body);

    Ok(res.to_vec())
}
