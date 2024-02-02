use std::collections::HashMap;

use crate::Rule;

use super::t::Type;
use super::method::Method;

#[derive(Debug)]
pub struct Structure<'a> {
    pub name: &'a str,
    pub members: HashMap<String, (Type, u8)>,
    pub methods: HashMap<String, Method<'a>>,
}

impl<'a> Structure<'a> {
    pub fn parse(pairs: &mut pest::iterators::Pairs<'a, Rule>) -> Self {
        let ident = pairs.next().unwrap().as_str();

        let mut members = HashMap::new();
        for member in pairs {
            let mut pairs = member.into_inner();
            let ident = pairs.next().unwrap().as_str();
            let t = Type::new(pairs.next().unwrap().as_str().parse().unwrap(), pairs.next().is_some());
            members.insert(ident.to_string(), (t, members.len() as u8));
        }

        Structure {
            name: ident,
            members,
            methods: HashMap::new(),
        }
    }
}
