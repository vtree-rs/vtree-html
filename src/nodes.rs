use std::collections::HashMap;
use std::convert::From;

#[derive(Debug, Clone, PartialEq)]
pub enum ParamValue {
    String(String),
    Str(&'static str),
    Bool(bool),
}

impl From<String> for ParamValue {
    fn from(v: String) -> ParamValue {
        ParamValue::String(v)
    }
}

impl From<&'static str> for ParamValue {
    fn from(v: &'static str) -> ParamValue {
        ParamValue::Str(v)
    }
}

impl From<bool> for ParamValue {
    fn from(v: bool) -> ParamValue {
        ParamValue::Bool(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Params {
    pub params: HashMap<String, ParamValue>,
}

impl Params {
    pub fn new() -> Params {
        Params {
            params: HashMap::new(),
        }
    }
}

define_nodes!(
    Text<String> {},
    Div<Params> {
        contents: mul AllNodes,
    },
    Span<Params> {
        contents: mul AllNodes,
    },
);
