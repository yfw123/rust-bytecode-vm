use std::convert::TryFrom;

use crate::agent::Agent;
use crate::code_object::CodeObject;
use crate::opcode::OpCode;
use crate::value::Value;

pub struct Interpreter<'a> {
    agent: &'a mut Agent<'a>,
    stack: Vec<Value<'a>>,
}

impl<'a> Interpreter<'a> {
    pub fn new(agent: &'a mut Agent<'a>) -> Interpreter<'a> {
        Interpreter {
            agent,
            stack: Vec::new(),
        }
    }

    pub fn evaluate(&mut self, code_object: CodeObject) -> Result<(), String> {
        let mut ip = 0;

        macro_rules! push { ($expr:expr) => { self.stack.push($expr) } }
        macro_rules! pop { () => { self.stack.pop().ok_or("Stack underflow")? } }
        macro_rules! next {
            () => {{
                let inst = code_object.instructions.get(ip);
                ip += 1;
                inst
            }};
            ($expr:expr) => {{
                let mut array = [0u8; $expr];

                for i in 0..$expr {
                    let result: Result<&u8, String> = next!()
                        .ok_or("Unexpected end of bytecode".into());
                    let n: u8 = *result?;
                    array[i] = n;
                }

                array
            }};
        }

        while let Some(instruction) = next!() {
            match OpCode::try_from(instruction)? {
                OpCode::Noop => {},

                OpCode::ConstInt => {
                    push!(Value::from(i64::from_le_bytes(next!(8))));
                },

                OpCode::ConstDouble => {
                    push!(Value::from(
                        f64::from_bits(u64::from_le_bytes(next!(8))),
                    ));
                },

                OpCode::ConstNull => {
                    push!(Value::Null);
                },

                OpCode::ConstTrue => {
                    push!(Value::from(true));
                },

                OpCode::ConstFalse => {
                    push!(Value::from(false));
                },

                OpCode::ConstString => {
                    let idx = usize::from_le_bytes(next!(8));
                    push!(Value::from(self.agent.string_table[idx]));
                },
            }
        }

        Ok(())
    }
}
