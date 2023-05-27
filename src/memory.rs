use std::collections::HashMap;
use anyhow::{Result, Context};

use crate::bytecode::ByteCode;

pub struct Memory {
    inner: [u8; 64000],
    definitions: HashMap<String, Vec<ByteCode>>,
}

impl Memory {
    pub fn new() -> Memory {
        let inner = [0; 64000];
        let definitions = HashMap::new();
        Memory { inner, definitions }
    }

    pub fn fetch(&self, idx: usize) -> Result<&u8> {
        self.inner.get(idx).context("no memory at index")
    }

    pub fn store(&mut self, idx: usize, val: u8) -> Result<u8> {
        let ptr = self.inner.get_mut(idx).context("no memory at index")?;
        *ptr = val;
        Ok(val)
    }

    pub fn define(&mut self, name: String, code: ByteCode) {
        let definitions = if let Some(d) = self.definitions.get_mut(&name.clone()) {
            d
        } else {
            let d = Vec::new();
            self.definitions.insert(name.clone(), d);
            self.definitions.get_mut(&name).unwrap()
        };
        definitions.push(code);
    }

    pub fn lookup(&self, name: String) -> Option<ByteCode> {
        if let Some(d) = self.definitions.get(&name) {
            Some(d.last().unwrap().to_owned())
        } else {
            None
        }
    }
}

