use crate::syntax_analyzer::{Expression, Function, Statement};

//
// ===== INTERMEDIATE REPRESENTATION (IR) STRUCTURES =====
//

// Each IR instruction represents a single "low-level" operation.
// This is similar to three-address code (used in compilers).

#[derive(Debug, Clone)]
pub enum IRInstr {
    Assign(String, IRValue),               // a = value
    BinaryOp(String, String, String, String), // result = left op right
    Return(String),
}

// Values used in IR instructions.
// They can be literals, variable names, or temporary registers.
#[derive(Debug, Clone)]
pub enum IRValue {
    Int(i64),
    Bool(bool),
    Str(String),
    Var(String),
    Temp(String), // temporary variable (like t1, t2)
}

// Holds state for generating IR — mainly the temp counter and the list of instructions.

pub struct IRGenerator {
    temp_counter: usize, //counter to create unique temps such as t1, t2, t3 ...
    code: Vec<IRInstr>, //List of the generated instructions
}

impl IRGenerator {
    pub fn new() -> Self {
        Self {
            temp_counter: 0,
            code: Vec::new(),
        }
    }

    //generates temp variable name
    fn new_temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("t{}", self.temp_counter)
    }

    //
    // ===== MAIN ENTRY POINT =====
    //
    // Converts a full parsed function into a vector of IR instructions.
    //
    pub fn generate_function(&mut self, func: &Function) -> Vec<IRInstr> {

        for stmt in &func.body {
            self.generate_statement(stmt);
        }
        //return full ir code
        self.code.clone()
    }

    //
    // ===== STATEMENT GENERATION =====
    //
    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            //handels var declarations
            Statement::VarDecl { name, value } => {
                let val = self.generate_expression(value);

                //adds assignment instruction
                self.code.push(IRInstr::Assign(name.clone(), val));
            }

            //handels return statements
            Statement::Return(expr) => {
                let val = self.generate_expression(expr);
                if let IRValue::Temp(t) | IRValue::Var(t) = val {
                    self.code.push(IRInstr::Return(t));
                } else {
                    // return literal directly
                    let tmp = self.new_temp();
                    self.code.push(IRInstr::Assign(tmp.clone(), val));
                    self.code.push(IRInstr::Return(tmp));
                }
            }

            // Expression statement: evaluate expression but discard result

            Statement::Expr(expr) => {
                self.generate_expression(expr);
            }
        }
    }

    fn generate_expression(&mut self, expr: &Expression) -> IRValue {
        match expr {
            // Literal values become immediate IR values

            Expression::Integer(n) => IRValue::Int(*n),
            Expression::Boolean(b) => IRValue::Bool(*b),
            Expression::String(s) => IRValue::Str(s.clone()),
            // Variable name -> IR variable reference

            Expression::Ident(name) => IRValue::Var(name.clone()),

            Expression::BinaryOp { left, op, right } => {
                //recursivly generate code for both sides
                let left_val = self.generate_expression(left);
                let right_val = self.generate_expression(right);
                let tmp = self.new_temp();

                let l = match left_val {
                    //if already a variable or temp then use it directly
                    IRValue::Var(ref v) | IRValue::Temp(ref v) => v.clone(),
                    IRValue::Int(_) | IRValue::Bool(_) | IRValue::Str(_) => {
                        let lit = self.new_temp();
                        self.code.push(IRInstr::Assign(lit.clone(), left_val));
                        lit
                    }
                };
                let r = match right_val {
                    IRValue::Var(ref v) | IRValue::Temp(ref v) => v.clone(),
                    IRValue::Int(_) | IRValue::Bool(_) | IRValue::Str(_) => {
                        let lit = self.new_temp();
                        self.code.push(IRInstr::Assign(lit.clone(), right_val));
                        lit
                    }
                };
                //add to the actaul binary operation instructions
                self.code.push(IRInstr::BinaryOp(tmp.clone(), l, op.clone(), r));
                IRValue::Temp(tmp)
            }
        }
    }
}

