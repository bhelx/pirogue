use crate::machine::{QUOTE, PUSH, SYMBOL, Instruction, StackItem};
use anyhow::{Result, bail};

pub type ByteCode = Vec<u8>;

pub struct ByteCodeBuilder {
    stack: Vec<ByteCodeBuilderInner>,
}

impl ByteCodeBuilder {
    pub fn new() -> ByteCodeBuilder {
        ByteCodeBuilder {
            stack: vec![ByteCodeBuilderInner::new()],
        }
    }

    pub fn push_val(&mut self, v: u8) {
        let builder = self.stack.last_mut().unwrap();
        builder.push_val(v);
    }

    pub fn push_symbol(&mut self, sym: &str) {
        let builder = self.stack.last_mut().unwrap();
        builder.push_symbol(sym);
    }

    pub fn write_quote(&mut self, bc: &ByteCode) {
        let builder = self.stack.last_mut().unwrap();
        builder.write_quote(bc);
    }

    pub fn push_quote(&mut self) {
        self.stack.push(ByteCodeBuilderInner::new());
    }

    pub fn pop_quote(&mut self) {
        let bytecode = self.stack.last().unwrap().bytecode().clone();
        self.stack.pop();
        let builder = self.stack.last_mut().unwrap();
        builder.write_quote(&bytecode);
    }

    pub fn bytecode(&self) -> &ByteCode {
        self.stack.last().unwrap().bytecode()
    }
}

struct ByteCodeBuilderInner {
    bytes: Vec<u8>,
}

impl ByteCodeBuilderInner {
    pub fn new() -> ByteCodeBuilderInner {
        ByteCodeBuilderInner {
            bytes: Vec::new(),
        }
    }

    pub fn push_val(&mut self, v: u8) {
        self.bytes.push(PUSH);
        self.bytes.push(v);
    }

    pub fn push_symbol(&mut self, sym: &str) {
        self.bytes.push(SYMBOL);
        for c in sym.chars() {
            self.bytes.push(c as u8);
        }
        self.bytes.push(0);
    }

    pub fn write_quote(&mut self, code: &ByteCode) {
        self.bytes.push(QUOTE);
        self.bytes.push(code.len() as u8);
        for c in code {
            self.bytes.push(*c);
        }
    }

    pub fn bytecode(&self) -> &ByteCode {
        &self.bytes
    }
}

pub struct ByteCodeParser<'a> {
    code: &'a ByteCode,
    idx: usize,
}

impl<'a> ByteCodeParser<'a> {
    pub fn new(code: &ByteCode) -> ByteCodeParser {
        ByteCodeParser {
            code,
            idx: 0,
        }
    }

    pub fn read_instruction(&mut self) -> Result<Instruction> {
        let opcode = self.consume_code(self.idx)?;
        let instr = match opcode {
            PUSH => self.read_push_instruction(),
            SYMBOL => self.read_symbol(),
            QUOTE => self.read_quote(),
            _ => bail!("Index {}: Unknown opcode {}", self.idx, opcode),
        }?;
        Ok(instr)
    }

    pub fn read_push_instruction(&mut self) -> Result<Instruction> {
        let val = self.consume_code(self.idx)?;
        Ok(Instruction::Push(StackItem::Val(val as i64)))
    }

    pub fn read_symbol(&mut self) -> Result<Instruction> {
        let mut s = String::new();
        while self.has_bytes() {
            let chr = self.consume_code(self.idx)?;
            if chr == 0 {
                break;
            }
            s.push(chr as char);
        }
        Ok(Instruction::Push(StackItem::Symbol(s)))
    }

    pub fn read_quote(&mut self) -> Result<Instruction> {
        let mut bc: ByteCode = Vec::new();
        let mut length = self.consume_code(self.idx)?;
        while self.has_bytes() && length > 0 {
            let byte = self.consume_code(self.idx)?;
            bc.push(byte);
            length -= 1;
        }
        Ok(Instruction::Push(StackItem::Quote(bc)))
    }

    pub fn consume_code(&mut self, idx: usize) -> Result<u8> {
        if let Some(opcode) = self.code.get(idx) {
            self.idx += 1;
            Ok(opcode.to_owned())
        } else {
            bail!("Could not find code at index")
        }
    }

    pub fn has_bytes(&self) -> bool {
        self.idx < self.code.len()
    }
}

pub fn bytecode_as_string(code: ByteCode) -> Result<String> {
    let mut parser = ByteCodeParser::new(&code);
    let mut parts = String::new();
    parts.push_str("[");
    while parser.has_bytes() {
        let i = parser.read_instruction()?;
        match i {
            Instruction::Push(item) => {
                match item.clone() {
                    StackItem::Val(v) => parts.push_str(v.to_string().as_str()),
                    StackItem::Symbol(s) => parts.push_str(s.as_str()),
                    StackItem::Quote(bc) => parts.push_str(bytecode_as_string(bc)?.as_str()),
                }
            }
        }
        if parser.has_bytes() { parts.push_str(" ") };
    }
    parts.push_str("]");
    Ok(parts)
}

