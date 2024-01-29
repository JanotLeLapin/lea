use crate::Rule;

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

    let mut res = bytes::BytesMut::new();
    res.put_u32(0xCAFEBABE);
    res.put_u16(0);
    res.put_u16(34);

    res.put_u16(5); // cp size

    res.put_u8(7);
    res.put_u16(2);
    res.put_u8(1);
    res.put_u16(module.len() as u16);
    res.put_slice(module.as_bytes());

    res.put_u8(7);
    res.put_u16(4);
    res.put_u8(1);
    res.put_u16(16);
    res.put_slice(b"java/lang/Object");

    res.put_u16(access::PUBLIC | access::SUPER);
    res.put_u16(1);
    res.put_u16(3);
    res.put_u64(0);

    Ok(res.to_vec())
}
