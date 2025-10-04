use std::collections::HashMap;
use crate::syntax_analyzer::{Expression, Function, Statement};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Str,
    Unknown,
}

pub struct SymbolTable {
    variables: HashMap<String, Type>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, ty: Type) -> Result<(), String> {
        if self.variables.contains_key(&name) {
            return Err(format!("Variable '{}' already declared", name));
        }
        self.variables.insert(name, ty);
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.variables.get(name)
    }
}


pub struct SemanticAnalyzer {
    symbols: SymbolTable,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self { symbols: SymbolTable::new() }
    }

    pub fn analyze_function(&mut self, func: &Function) -> Result<(), String> {
        for stmt in &func.body {
            self.analyze_statement(stmt)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), String> {
        match stmt {
            Statement::VarDecl { name, value } => {
                let ty = self.analyze_expression(value)?;
                self.symbols.insert(name.clone(), ty)?;
            }
            Statement::Return(expr) => {
                let _ty = self.analyze_expression(expr)?;
                // later: check against function return type
            }
            Statement::Expr(expr) => {
                self.analyze_expression(expr)?;
            }
        }
        Ok(())
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Result<Type, String> {
        match expr {
            Expression::Integer(_) => Ok(Type::Int),
            Expression::Boolean(_) => Ok(Type::Bool),
            Expression::String(_) => Ok(Type::Str),
            Expression::Ident(name) => {
                self.symbols
                    .lookup(name)
                    .cloned()
                    .ok_or_else(|| format!("Use of undeclared variable '{}'", name))
            }
            Expression::BinaryOp { left, op, right } => {
                let left_ty = self.analyze_expression(left)?;
                let right_ty = self.analyze_expression(right)?;

                if left_ty != right_ty {
                    return Err(format!(
                        "Type mismatch in binary op '{}': {:?} vs {:?}",
                        op, left_ty, right_ty
                    ));
                }

                match op.as_str() {
                    "+" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok(Type::Int)
                        } else if left_ty == Type::Str && right_ty == Type::Str {
                            Ok(Type::Str)
                        } else {
                            Err(format!(
                                "Operator '+' not supported between {:?} and {:?}",
                                left_ty, right_ty
                            ))
                        }
                    }
                    "-" | "*" | "/" => {
                        if left_ty == Type::Int && right_ty == Type::Int {
                            Ok(Type::Int)
                        } else {
                            Err(format!("Operator '{}' not supported for {:?}", op, left_ty))
                        }
                    }
                    "==" | "!=" => {
                        if left_ty == right_ty {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "Cannot compare values of different types: {:?} vs {:?}",
                                left_ty, right_ty
                            ))
                        }
                    }
                    _ => Err(format!("Unknown operator '{}'", op)),
                }

            }
        }
    }
}
