use crate::Rule;

use bytes::BufMut;

pub fn compile_method<'a>(pairs: &mut pest::iterators::Pairs<'a, Rule>, cp: &mut crate::compiler::constant_pool::ConstantPool) -> Vec<u8> {
    let mut buf = bytes::BytesMut::new();
    buf.put_u16(1 | 8);
    buf.put_u16(cp.insert_utf8(pairs.next().unwrap().as_str().to_string()));

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

    let (ret_type, block) =
        if next.as_rule() == Rule::block { ("V", next) }
        else {
            (crate::compiler::parse_type(next.as_str()), pairs.next().unwrap())
        };
    buf.put_u16(cp.insert_utf8(format!("({}){}", params.join(""), ret_type)));

    let attribute = super::attribute::compile_code(&mut block.into_inner(), cp);

    buf.put_u16(0);
    buf.to_vec()
}
