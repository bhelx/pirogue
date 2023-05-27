use crate::bytecode::{ByteCode, ByteCodeParser, bytecode_as_string, ByteCodeBuilder};
use crate::memory::Memory;
use std::fmt;
use anyhow::{Result, bail, Context};

pub const PUSH: u8 = 1;
pub const SYMBOL: u8 = 2;
pub const QUOTE: u8 = 3;

#[derive(Clone)]
pub enum StackItem {
    Val(i64),
    Symbol(String),
    Quote(ByteCode),
}

impl fmt::Debug for StackItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackItem::Val(v) => write!(f, "{}", v),
            StackItem::Symbol(s) => write!(f, "{}", s),
            StackItem::Quote(bc) => write!(f, "<bytecode:{}>", bc.len()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(StackItem)
}

pub struct Vm {
    memory: Memory,
    stack: Vec<StackItem>,
}

impl Vm {
    pub fn new() -> Vm {
        Vm {
            memory: Memory::new(),
            stack: Vec::new(),
        }
    }

    pub fn eval(&mut self, bytecode: &ByteCode) -> Result<()> {
        let mut parser = ByteCodeParser::new(bytecode);
        while parser.has_bytes() {
            let i = parser.read_instruction()?;
            match i {
                Instruction::Push(item) => {
                    match item.clone() {
                        StackItem::Val(_) => self.stack.push(item),
                        StackItem::Symbol(s) => {
                            if !self.apply_symbol(s)? {
                                self.stack.push(item)
                            }
                        },
                        StackItem::Quote(_) => {
                            self.stack.push(item)
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn apply_symbol(&mut self, s: String) -> Result<bool> {
        match s.as_str() {
            "+" => {
                let b = self.pop_val()?;
                let a = self.pop_val()?;
                self.stack.push(StackItem::Val(a + b));
                Ok(true)
            },
            "-" => {
                let b = self.pop_val()?;
                let a = self.pop_val()?;
                self.stack.push(StackItem::Val(a - b));
                Ok(true)
            },
            "*" => {
                let b = self.pop_val()?;
                let a = self.pop_val()?;
                self.stack.push(StackItem::Val(a * b));
                Ok(true)
            },
            "/" => {
                let b = self.pop_val()?;
                let a = self.pop_val()?;
                self.stack.push(StackItem::Val(a / b));
                Ok(true)
            },
            "=" => {
                let b = self.pop_val()?;
                let a = self.pop_val()?;
                let mut builder = ByteCodeBuilder::new();
                if a == b {
                    builder.push_symbol("true");
                    self.stack.push(StackItem::Quote(builder.bytecode().to_owned()));
                } else {
                    builder.push_symbol("false");
                    self.stack.push(StackItem::Quote(builder.bytecode().to_owned()));
                }
                Ok(true)
            },
            "i" => {
                let v = self.pop()?;
                match v.clone() {
                    StackItem::Val(_) => self.stack.push(v),
                    StackItem::Symbol(_) => self.stack.push(v),
                    StackItem::Quote(bytecode) => self.eval(&bytecode)?,
                }
                Ok(true)
            },
            "@" => {
                let ptr = self.pop_val()?.to_owned();
                let val = self.memory.fetch(ptr as usize)?.to_owned();
                self.stack.push(StackItem::Val(val as i64));
                Ok(true)
            },
            "!" => {
                let ptr = self.pop_val()?.to_owned();
                let val = self.pop_val()?.to_owned();
                let _ = self.memory.store(ptr as usize, val as u8)?;
                Ok(true)
            },
            "." => {
                let v = self.pop()?;
                print!("{:#?} ", v);
                Ok(true)
            },
            "zap" => {
                let _ = self.pop()?;
                Ok(true)
            },
            "dup" => {
                let v = self.pop()?;
                self.stack.push(v.clone());
                self.stack.push(v);
                Ok(true)
            },
            "swap" => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.stack.push(a);
                self.stack.push(b);
                Ok(true)
            },
            "rot" => {
                let a = self.pop()?;
                let b = self.pop()?;
                let c = self.pop()?;
                self.stack.push(b);
                self.stack.push(a);
                self.stack.push(c);
                Ok(true)
            },
            "over" => {
                let a = self.pop()?;
                let b = self.pop()?;
                self.stack.push(b.clone());
                self.stack.push(a);
                self.stack.push(b);
                Ok(true)
            },
            "dip" => {
                let a = self.pop_quote()?;
                let b = self.pop_quote()?;
                self.eval(&a)?;
                self.stack.push(StackItem::Quote(b));
                Ok(true)
            },
            "cat" => {
                let mut a = self.pop_quote()?;
                let mut b = self.pop_quote()?;
                b.append(&mut a);
                self.stack.push(StackItem::Quote(b));
                Ok(true)
            },
            "cons" => {
                let a = self.pop_quote()?;
                let b = self.pop_quote()?;
                // TODO - this kind of byecode surgery is fragile
                let mut c = ByteCode::new();
                let len_b = b.len() as u8;
                c.push(QUOTE);
                c.push(len_b);
                c.extend(&b);
                c.extend(&a);
                self.stack.push(StackItem::Quote(c));
                Ok(true)
            },
            "unit" => {
                let a = self.pop_quote()?;
                // TODO - a little sketchy but okay, logic should be moved
                let mut builder = ByteCodeBuilder::new();
                builder.write_quote(&a);
                self.stack.push(StackItem::Quote(builder.bytecode().to_owned()));
                Ok(true)
            },
            ".s" => {
                self.print()?;
                Ok(true)
            },
            "define" => {
                let name = self.pop_symbol()?;
                let code = self.pop_quote()?;
                self.memory.define(name, code);
                Ok(true)
            },
            "<" => {
                let b = self.pop_val()?;
                let a = self.pop_val()?;
                let r = if a < b { "true" } else { "false" };
                self.stack.push(StackItem::Symbol(r.to_string()));
                Ok(true)
            },
            sym => {
                if let Some(code) = self.memory.lookup(sym.to_string()) {
                    self.eval(&code)?;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub fn pop(&mut self) -> Result<StackItem> {
        self.stack.pop().context("Stack Underflow")
    }

    pub fn pop_val(&mut self) -> Result<i64> {
        if let StackItem::Val(v) = self.pop()? {
            Ok(v)
        } else {
            bail!("TOS is not a val")
        }
    }

    pub fn pop_symbol(&mut self) -> Result<String> {
        if let StackItem::Symbol(v) = self.pop()? {
            Ok(v)
        } else {
            bail!("TOS is not a symbol")
        }
    }

    pub fn pop_quote(&mut self) -> Result<ByteCode> {
        if let StackItem::Quote(bc) = self.pop()? {
            Ok(bc)
        } else {
            bail!("TOS is not a quote")
        }
    }

    pub fn print(&self) -> Result<()> {
        let mut parts = String::new();
        let mut i = 0;
        for s in &self.stack {
            match s {
                StackItem::Quote(bc) => parts.push_str(bytecode_as_string(bc.to_owned())?.as_str()),
                StackItem::Val(v) => parts.push_str(v.to_string().as_str()),
                StackItem::Symbol(s) => parts.push_str(s.as_str()),
            }
            i += 1;
            if i < self.stack.len() { parts.push_str(" ") };
        }
        println!("[{}]", parts);
        Ok(())
    }
}

