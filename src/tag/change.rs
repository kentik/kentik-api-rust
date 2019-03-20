use std::collections::HashMap;
use std::net::IpAddr;
use Rules::*;

#[derive(Eq, PartialEq, Debug)]
pub enum Change {
    Upsert(Upsert),
    Delete(Delete),
}

#[derive(Eq, PartialEq, Debug)]
pub struct Upsert(String, Values);

#[derive(Eq, PartialEq, Debug)]
pub struct Delete(String, String);

#[derive(Eq, PartialEq, Debug)]
pub struct Values(HashMap<String, Rules>);

#[derive(Eq, PartialEq, Debug)]
pub enum Rules {
    One(Rule),
    All(Vec<Rule>),
    Empty,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Rule {
    IP(IpAddr),
    Port(u16),
 }

pub fn upsert(column: &str) -> Upsert {
    Upsert(column.to_string(), Values(HashMap::new()))
}

impl Upsert {
    pub fn value(&mut self, value: &str) -> &mut Rules {
        let Upsert(_, Values(map)) = self;
        map.entry(value.to_string()).or_insert(Empty)
    }
}

impl Rules {
    pub fn when(&mut self, rule: Rule) -> &mut Self {
        *self = One(rule);
        self
    }

    pub fn and(&mut self, rule: Rule) -> &mut Self {
        match self {
            One(one) => *self = All(vec![*one, rule]),
            All(vec) => vec.push(rule),
            Empty    => *self = All(vec![rule]),
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        *self == Empty
    }
}

impl<T: Into<Values>> From<(&str, T)> for Upsert {
    fn from((column, values): (&str, T)) -> Self {
        Upsert(column.to_owned(), values.into())
    }
}

impl<T: Into<Rules>> From<(&str, T)> for Values {
    fn from((value, rules): (&str, T)) -> Self {
        let mut map = HashMap::new();
        map.insert(value.to_owned(), rules.into());
        Values(map)
    }
}

impl<T: Into<Rules> + Copy> From<&[(&str, T)]> for Values {
    fn from(values: &[(&str, T)]) -> Self {
        Values(values.iter().map(|&(value, rules)| {
            (value.to_owned(), rules.into())
        }).collect())
    }
}

impl From<Rule> for Rules {
    fn from(rule: Rule) -> Self {
        Rules::One(rule)
    }
}

impl Into<(String, Vec<super::Upsert>)> for Upsert {
    fn into(self) -> (String, Vec<super::Upsert>) {
        (self.0, self.1.into())
    }
}

impl Into<Vec<super::Upsert>> for Values {
    fn into(self) -> Vec<super::Upsert> {
        self.0.into_iter().flat_map(|(value, rules)| {
            collect(value, rules)
        }).collect()
    }
}

fn collect(value: String, rules: Rules) -> Vec<super::Upsert> {
    let mut vec = Vec::new();
    match rules {
        One(rule)  => vec.push(super::Upsert::Small(one(value, rule))),
        All(rules) => vec.push(super::Upsert::Large(all(value, rules))),
        Empty      => (),
    }
    vec
}

fn one(value: String, src: Rule) -> super::Small {
    super::Small{value, criteria: (src.into(),)}
}

fn all(value: String, src: Vec<Rule>) -> super::Large {
    let mut rules = super::Rules::default();
    for rule in src {
        match rule {
            Rule::IP(ip)     => rules.addr.push(ip.to_string()),
            Rule::Port(port) => rules.port.push(port.to_string()),
        }
    }
    super::Large{value, criteria: vec![rules]}
}

impl Into<super::Rule> for Rule {
    fn into(self) -> super::Rule {
        let mut rules = super::Rule::default();
        match self {
            Rule::IP(ip)     => rules.addr = Some((ip.to_string(),)),
            Rule::Port(port) => rules.port = Some((port.to_string(),)),
        };
        rules
    }
}

#[cfg(test)]
mod test {
    use super::{*, Rule::*};

    #[test]
    fn upsert_basic() {
        let mut change = upsert("c_foo");
        change.value("bar").when(Port(22));

        assert_eq!(change, ("c_foo", ("bar", Port(22))).into())
    }

    #[test]
    fn upsert_multiple_values() {
        let mut change = upsert("c_foo");
        change.value("bar").when(Port(22));
        change.value("baz").when(Port(23));

        assert_eq!(change, ("c_foo", &[
            ("bar", Port(22)),
            ("baz", Port(23)),
        ][..]).into());
    }

    #[test]
    fn upsert_and_values() {
        let ip = "10.0.0.1".parse().unwrap();

        let mut change = upsert("c_foo");
        change.value("bar").when(IP(ip)).and(Port(22));

        assert_eq!(change, ("c_foo", ("bar", All(vec![
            IP(ip),
            Port(22),
        ]))).into());
    }
}
