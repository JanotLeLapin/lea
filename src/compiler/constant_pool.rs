use std::collections::HashMap;

use bytes::{BufMut, BytesMut};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Ref { Field, Method, InterfaceMethod }

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Constant {
    UTF8(String),
    Class(u16),
    String(u16),
    NameAndType(u16, u16),
    Ref(Ref, u16, u16),
}

#[derive(Debug)]
pub struct ConstantPool {
    count: u16,
    pool: HashMap<Constant, u16>,
}

impl ConstantPool {
    pub fn new() -> Self {
        Self {
            count: 0,
            pool: HashMap::new(),
        }
    }

    fn get_or_insert(&mut self, constant: Constant) -> u16 {
        if let Some(res) = self.pool.get(&constant) {
            *res
        } else {
            self.count += 1;
            self.pool.insert(constant, self.count);
            self.count
        }
    }

    pub fn insert_utf8(&mut self, value: String) -> u16 {
        self.get_or_insert(Constant::UTF8(value))
    }

    pub fn insert_class(&mut self, class: String) -> u16 {
        let idx = self.insert_utf8(class);
        self.get_or_insert(Constant::Class(idx))
    }

    pub fn insert_string(&mut self, string: String) -> u16 {
        let idx = self.insert_utf8(string);
        self.get_or_insert(Constant::String(idx))
    }

    pub fn insert_name_and_type(&mut self, name: String, t: String) -> u16 {
        let name = self.insert_utf8(name);
        let t = self.insert_utf8(t);
        self.get_or_insert(Constant::NameAndType(name, t))
    }

    pub fn insert_ref(&mut self, reference: Ref, class: String, name: String, t: String) -> u16 {
        let class_idx = self.insert_class(class);
        let idx = self.insert_name_and_type(name, t);
        self.get_or_insert(Constant::Ref(reference, class_idx, idx))
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u16(self.count + 1);

        let mut items = self.pool.iter().collect::<Vec<_>>();
        items.sort_by(|a, b| a.1.cmp(b.1));
        for (item, _) in items {
            match item {
                Constant::UTF8(value) => {
                    buf.put_u8(1);
                    buf.put_u16(value.len() as u16);
                    buf.put_slice(value.as_bytes());
                },
                Constant::Class(class) => {
                    buf.put_u8(7);
                    buf.put_u16(*class);
                },
                Constant::String(string) => {
                    buf.put_u8(8);
                    buf.put_u16(*string);
                },
                Constant::NameAndType(var, t) => {
                    buf.put_u8(12);
                    buf.put_u16(*var);
                    buf.put_u16(*t);
                },
                Constant::Ref(reference, class, name_type) => {
                    buf.put_u8(match reference {
                        Ref::Field => 9,
                        Ref::Method => 10,
                        Ref::InterfaceMethod => 11,
                    });
                    buf.put_u16(*class);
                    buf.put_u16(*name_type);
                },
            }
        }
        buf.to_vec()
    }
}
