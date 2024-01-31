use crate::Rule;

use bytes::BufMut;

#[derive(Debug)]
pub struct Method {
    access_flags: u16,
    name: String,
    descriptor: String,
}

impl Method {
    pub fn compile<'a>(pairs: &mut pest::iterators::Pairs<'a, Rule>) -> Self {
        let ident = pairs.next().unwrap().as_str();

        let mut params = vec![];
        let next = loop {
            let node = pairs.next().unwrap();
            if node.as_rule() != Rule::parameter { break node; }

            let mut pairs = node.into_inner();
            let _ = pairs.next();
            let t = crate::compiler::parse_type(pairs.next().unwrap().as_str());
            let is_array = pairs.next().is_some();

            if is_array { params.push(format!("[{}", t)) }
            else { params.push(t.to_string()) }
        };

        let ret_type =
            if next.as_rule() == Rule::block { "V" }
            else { crate::compiler::parse_type(next.as_str()) };

        Self {
            access_flags: 1 | 8,
            name: ident.to_string(),
            descriptor: format!("({}){}", params.join(""), ret_type),
        }
    }

    pub fn serialize(&self, cp: &mut crate::compiler::constant_pool::ConstantPool) -> Vec<u8> {
        let mut buf = bytes::BytesMut::new();
        buf.put_u16(self.access_flags);
        buf.put_u16(cp.insert_utf8(self.name.clone()));
        buf.put_u16(cp.insert_utf8(self.descriptor.clone()));
        buf.put_u16(0);
        buf.to_vec()
    }
}
