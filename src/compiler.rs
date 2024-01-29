use crate::Rule;

use bytes::{Buf, BufMut, BytesMut};

enum Constant<'a> {
    Class(u16),
    UTF8(&'a str),
}

trait ClassFile {
    fn put_constant(&mut self, constant: Constant<'_>);
}

impl ClassFile for BytesMut {
    fn put_constant(&mut self, constant: Constant<'_>) {
        use Constant::*;

        match constant {
            Class(idx) => {
                self.put_u8(7);
                self.put_u16(idx);
            },
            UTF8(slice) => {
                self.put_u8(1);
                self.put_u16(slice.len() as u16);
                self.put_slice(slice.as_bytes());
            }
        }
    }
}

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
    use Constant::*;

    let module = match ast.next() {
        Some(pair) => match pair.as_rule() {
            Rule::module_statement => Some(pair.into_inner().as_str()),
            _ => None
        },
        _ => None,
    }.ok_or(ExpectedModule)?;

    let cp = vec![
        Class(2), UTF8(module),
        Class(4), UTF8("java/lang/Object"),
    ];

    let mut res = bytes::BytesMut::new();
    res.put_u32(0xCAFEBABE);
    res.put_u16(0);
    res.put_u16(52);

    res.put_u16(cp.len() as u16 + 1);
    for constant in cp { res.put_constant(constant); };

    res.put_u16(access::PUBLIC | access::SUPER);
    res.put_u16(1);
    res.put_u16(3);
    res.put_u64(0);

    Ok(res.to_vec())
}
