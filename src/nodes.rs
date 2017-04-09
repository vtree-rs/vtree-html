use std::collections::HashMap;
use std::convert::From;
use vtree_macros::define_nodes;

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

#[macro_export]
macro_rules! html_nodes {
    ($e:ident) => {
        $e!{
            (0, Div, div),
            (1, Span, span),
            (2, Ul, ul),
            (3, Li, li),
        }
    };
}

macro_rules! gen_html_nodes_array {
    ($(($id:expr, $name_up:ident, $name_low:ident),)*) => {
        [
            $(
                ($id, stringify!($name_up), stringify!($name_low)),
            )*
        ]
    };
}

pub fn html_nodes_iter() -> impl Iterator<Item=&'static (usize, &'static str, &'static str)> {
    static ARR: &'static [(usize, &'static str, &'static str)] = &html_nodes!(gen_html_nodes_array);
    ARR.iter()
}

macro_rules! gen_node_defs {
    ($(($id:expr, $name_up:ident, $name_low:ident),)*) => {
        define_nodes!(
            Text<String>,
            $(
                $name_up<Params> {
                    contents: mul AllNodes,
                },
            )*
        );
    };
}

html_nodes!(gen_node_defs);
