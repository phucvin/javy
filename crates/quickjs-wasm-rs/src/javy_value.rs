// use std::{collections::HashMap};

use std::io::Bytes;

// Should this type be in a completely separate crate if we plan to have multiple JS engines?
// That way the spidermonkey engine can also use to serialize to their internal types
pub enum JavyValue {
    Undefined,
    Null,
    Bool(bool),
    Int(i32), // do we need to support i8..i64?
    Float(f64),
    String(String),
    Bytecode(Vec<u8>), // Thoughts on having this dedicated type for Bytecode?
    // Array(Vec<JavyValue>),
    // Object(HashMap<String, JavyValue>),
}

impl JavyValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JavyValue::String(ref s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn into_string(self) -> Option<String> {
        match self {
            JavyValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn from_bytecode(bytecode: &[u8]) -> Self {
        JavyValue::Bytecode(bytecode.to_vec())
    }
}

