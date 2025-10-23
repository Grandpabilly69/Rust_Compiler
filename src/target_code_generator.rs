// target_code_generator.rs
use std::collections::HashMap;

use crate::intermediate_code_generator::{IRInstr, IRValue}; // adjust path if needed

// ===== VM instruction set (your existing opcodes, unchanged) =====
#[derive(Debug, Clone)]
pub enum VMInstr {
    PushInt(i64),
    PushBool(bool),
    PushStr(String),
    Load(String),   // push variable value onto stack
    Store(String),  // pop stack, store into variable
    Add,
    Sub,
    Mul,
    Div,
    Concat, // string concatenation
    Ret,    // return with top-of-stack
}

// ===== runtime values on the VM stack =====
#[derive(Debug, Clone)]
pub enum VMValue {
    Int(i64),
    Bool(bool),
    Str(String),
}

// ===== a call frame =====
// each frame owns its own local variables map.
// for now we keep it simple: no return-ip / caller state because
// we are executing a single top-level function body. When adding calls,
// you'll add return_ip / caller stacks here.
#[derive(Debug, Clone)]
pub struct Frame {
    pub locals: HashMap<String, VMValue>,
}

impl Frame {
    pub fn new() -> Self {
        Self { locals: HashMap::new() }
    }
}

// ===== a program (linear list of VM instructions) =====
#[derive(Debug, Clone)]
pub struct VMProgram {
    pub instrs: Vec<VMInstr>,
}

// ===== the VM itself =====
pub struct VM {
    stack: Vec<VMValue>,     // evaluation stack
    frames: Vec<Frame>,      // call stack (frame 0 is global)
}

impl VM {
    /// Create a new VM with an empty global frame
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            frames: vec![Frame::new()], // start with a global frame
        }
    }

    /// Helper: push a value onto the evaluation stack
    fn push(&mut self, v: VMValue) {
        self.stack.push(v);
    }

    /// Helper: pop a value from the evaluation stack
    fn pop(&mut self) -> VMValue {
        self.stack.pop().expect("VM stack underflow")
    }

    /// Helper: store a variable in the current frame
    fn set_var(&mut self, name: &str, val: VMValue) {
        let frame = self.frames.last_mut().expect("No call frame");
        frame.locals.insert(name.to_string(), val);
    }

    /// Helper: load a variable from the current frame
    fn get_var(&self, name: &str) -> Option<VMValue> {
        let frame = self.frames.last().expect("No call frame");
        frame.locals.get(name).cloned()
    }

    /// Execute a VMProgram and return an optional VMValue from the first Ret.
    /// This is a simple interpreter loop. It returns the top-of-stack value
    /// when it sees a `Ret` instruction.
    pub fn run(&mut self, program: &VMProgram) -> Option<VMValue> {
        for instr in &program.instrs {
            match instr {
                VMInstr::PushInt(n) => self.push(VMValue::Int(*n)),
                VMInstr::PushBool(b) => self.push(VMValue::Bool(*b)),
                VMInstr::PushStr(s) => self.push(VMValue::Str(s.clone())),

                VMInstr::Load(name) => {
                    match self.get_var(name) {
                        Some(v) => self.push(v),
                        None => panic!("Undefined variable '{}' on Load", name),
                    }
                }

                VMInstr::Store(name) => {
                    let v = self.pop();
                    self.set_var(name, v);
                }

                // arithmetic + concat
                VMInstr::Add => {
                    let r = self.pop();
                    let l = self.pop();
                    match (l, r) {
                        (VMValue::Int(a), VMValue::Int(b)) => self.push(VMValue::Int(a + b)),
                        (VMValue::Str(a), VMValue::Str(b)) => {
                            // String + String -> String
                            self.push(VMValue::Str(a + &b));
                        }
                        // optionally allow mixing string+int via format!:
                        (VMValue::Str(a), VMValue::Int(b)) => {
                            self.push(VMValue::Str(format!("{}{}", a, b)));
                        }
                        (VMValue::Int(a), VMValue::Str(b)) => {
                            self.push(VMValue::Str(format!("{}{}", a, b)));
                        }
                        _ => panic!("Type error in Add"),
                    }
                }

                VMInstr::Sub => {
                    let r = self.pop();
                    let l = self.pop();
                    match (l, r) {
                        (VMValue::Int(a), VMValue::Int(b)) => self.push(VMValue::Int(a - b)),
                        _ => panic!("Type error in Sub"),
                    }
                }

                VMInstr::Mul => {
                    let r = self.pop();
                    let l = self.pop();
                    match (l, r) {
                        (VMValue::Int(a), VMValue::Int(b)) => self.push(VMValue::Int(a * b)),
                        _ => panic!("Type error in Mul"),
                    }
                }

                VMInstr::Div => {
                    let r = self.pop();
                    let l = self.pop();
                    match (l, r) {
                        (VMValue::Int(_), VMValue::Int(0)) => panic!("Division by zero"),
                        (VMValue::Int(a), VMValue::Int(b)) => self.push(VMValue::Int(a / b)),
                        _ => panic!("Type error in Div"),
                    }
                }

                VMInstr::Concat => {
                    let r = self.pop();
                    let l = self.pop();
                    match (l, r) {
                        (VMValue::Str(a), VMValue::Str(b)) => self.push(VMValue::Str(a + &b)),
                        _ => panic!("Type error in Concat"),
                    }
                }

                // Return: pop top-of-stack and return it as program result
                VMInstr::Ret => {
                    return Some(self.pop());
                }
            }
        }

        None
    }
}

// ===== Lowering from IR to VMProgram (simple deterministic lowering) =====
pub fn lower_ir_to_vm(ir: &[IRInstr]) -> VMProgram {
    let mut instrs: Vec<VMInstr> = Vec::new();

    for instr in ir {
        match instr {
            IRInstr::Assign(target, value) => match value {
                IRValue::Int(n) => {
                    instrs.push(VMInstr::PushInt(*n));
                    instrs.push(VMInstr::Store(target.clone()));
                }
                IRValue::Bool(b) => {
                    instrs.push(VMInstr::PushBool(*b));
                    instrs.push(VMInstr::Store(target.clone()));
                }
                IRValue::Str(s) => {
                    instrs.push(VMInstr::PushStr(s.clone()));
                    instrs.push(VMInstr::Store(target.clone()));
                }
                IRValue::Var(v) | IRValue::Temp(v) => {
                    // copy from another variable/temp
                    instrs.push(VMInstr::Load(v.clone()));
                    instrs.push(VMInstr::Store(target.clone()));
                }
            },

            IRInstr::BinaryOp(result, left, op, right) => {
                // load left then right (order chosen here)
                instrs.push(VMInstr::Load(left.clone()));
                instrs.push(VMInstr::Load(right.clone()));

                match op.as_str() {
                    "+" => instrs.push(VMInstr::Add),
                    "-" => instrs.push(VMInstr::Sub),
                    "*" => instrs.push(VMInstr::Mul),
                    "/" => instrs.push(VMInstr::Div),
                    _ => instrs.push(VMInstr::Add), // fallback; ideally handle other ops
                }

                instrs.push(VMInstr::Store(result.clone()));
            }

            IRInstr::Return(name) => {
                instrs.push(VMInstr::Load(name.clone()));
                instrs.push(VMInstr::Ret);
            }
        }
    }

    VMProgram { instrs }
}

// ===== convenience: run IR through lowering and the VM =====
pub fn run_ir_with_vm(ir: &[IRInstr]) -> Option<VMValue> {
    let prog = lower_ir_to_vm(ir);
    let mut vm = VM::new();
    vm.run(&prog)
}
