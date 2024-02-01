use std::collections::HashMap;

use crate::Rule;

use bytes::BufMut;

#[derive(Debug)]
pub struct Method<'a> {
    pub name: &'a str,
    descriptor: String,
    args: HashMap<String, (String, u16)>,
    code: pest::iterators::Pairs<'a, Rule>,
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
            code: block.into_inner(),
        }
    }

    pub fn compile(&mut self, cp: &mut crate::compiler::constant_pool::ConstantPool) -> Vec<u8> {
        let mut buf = bytes::BytesMut::new();
        buf.put_u16(1 | 8);
        buf.put_u16(cp.insert_utf8(self.name.to_string()));
        buf.put_u16(cp.insert_utf8(self.descriptor.clone()));

        let attribute = super::attribute::compile_code(&mut self.code, cp);

        buf.put_u16(1);
        buf.put_slice(&attribute);
        buf.to_vec()
    }
}
