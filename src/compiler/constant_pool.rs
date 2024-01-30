use std::collections::HashMap;

use bytes::{BufMut, BytesMut};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Constant<'a> {
    UTF8(&'a str),
    Class(u16),
}

#[derive(Debug)]
pub struct ConstantPool<'a> {
    count: u16,
    pool: HashMap<Constant<'a>, u16>,
}

impl<'a> ConstantPool<'a> {
    pub fn new() -> Self {
        Self {
            count: 0,
            pool: HashMap::new(),
        }
    }

    fn get_or_insert(&mut self, constant: Constant<'a>) -> u16 {
        if let Some(res) = self.pool.get(&constant) {
            *res
        } else {
            self.count += 1;
            self.pool.insert(constant, self.count);
            self.count
        }
    }

    pub fn insert_utf8(&mut self, value: &'a str) -> u16 {
        self.get_or_insert(Constant::UTF8(value))
    }

    pub fn insert_class(&mut self, class: &'a str) -> u16 {
        let idx = self.insert_utf8(class);
        self.get_or_insert(Constant::Class(idx))
    }

    pub fn serialize(&self) -> Vec<u8> {
        use Constant::*;

        let mut buf = BytesMut::new();
        buf.put_u16(self.count + 1);

        let mut items = self.pool.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.1.cmp(b.1));
        for (item, _) in items {
            match item {
                UTF8(value) => {
                    buf.put_u8(1);
                    buf.put_u16(value.len() as u16);
                    buf.put_slice(value.as_bytes());
                },
                Class(class) => {
                    buf.put_u8(7);
                    buf.put_u16(*class);
                },
            }
        }
        buf.to_vec()
    }
}
