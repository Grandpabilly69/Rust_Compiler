use crate::lex_layer::{LiteralType, Token};
//There is an error where it is expecting a delimeter but finds an identifier.
//The fix will be made at a later day

//AST Types start
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    VarDecl { name: String, value: Expression },
    Expr(Expression),
    Return(Expression),
}


#[derive(Debug)]
pub enum Expression {
    Integer(i64),
    Boolean(bool),
    String(String),
    Ident(String),
    BinaryOp {
        left: Box<Expression>,
        op: String,
        right: Box<Expression>,
    },
}
//AST types end


//Parser Struct start
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    fn peek_raw(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn advance_raw(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.current);
        if tok.is_some() {
            self.current += 1;
        }
        tok
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), String> {
        match self.advance() {
            Some(Token::Keyword(s)) if s == kw => Ok(()),
            other => Err(format!("Expected keyword '{}', found {:?}", kw, other)),
        }
    }

    fn expect_operator(&mut self, op: &str) -> Result<(), String> {
        match self.advance() {
            Some(Token::Operator(s)) if s == op => Ok(()),
            other => Err(format!("Expected operator '{}', found {:?}", op, other)),
        }
    }
    fn expect_delim_raw(&mut self, ch: char) -> Result<(), String> {
        while let Some(tok) = self.tokens.get(self.current) {
            match tok {
                Token::Whitespace | Token::Comment => self.current += 1, // skip
                Token::Delimiter(c) if *c == ch => {
                    self.current += 1;
                    return Ok(());
                }
                other => return Err(format!("Expected delimiter '{}', found {:?}", ch, other)),
            }
        }
        Err(format!("Expected delimiter '{}', found end of input", ch))
    }
}
//Parser struct end


//Parse a func start
impl<'a> Parser<'a> {
    pub fn parse_function(&mut self) -> Result<Function, String> {
        self.expect_keyword("func")?;

        let name = match self.advance() {
            Some(Token::Identifier(s)) => s.clone(),
            other => return Err(format!("Expected function name, found {:?}", other)),
        };

        self.expect_delim('(')?;
        let mut params = Vec::new();
        loop {
            match self.peek() {
                Some(Token::Identifier(s)) => {
                    params.push(s.clone());
                    self.advance();
                    if let Some(Token::Delimiter(',')) = self.peek() {
                        self.advance();
                    }
                }
                Some(Token::Delimiter(')')) => {
                    self.advance();
                    break;
                }
                other => return Err(format!("Unexpected token in parameters: {:?}", other)),
            }
        }

        self.expect_delim('{')?;
        let body = self.parse_statements()?;
        self.expect_delim('}')?;

        Ok(Function { name, params, body })
    }
}
//Parse a func end

//parse statements start
impl<'a> Parser<'a> {
    fn parse_statements(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = Vec::new();
        while let Some(tok) = self.peek() {
            match tok {
                Token::Keyword(s) if s == "var" => stmts.push(self.parse_var_decl()?),
                Token::Keyword(s) if s == "return" => stmts.push(self.parse_return()?),
                Token::Delimiter('}') => break,
                _ => {
                    let expr = self.parse_expression()?;
                    self.expect_delim(';')?;
                    stmts.push(Statement::Expr(expr));
                }
            }
        }
        Ok(stmts)
    }


    fn parse_var_decl(&mut self) -> Result<Statement, String> {
        self.expect_keyword("var")?;

        let name = match self.advance() {
            Some(Token::Identifier(s)) => s.clone(),
            other => return Err(format!("Expected identifier after 'var', found {:?}", other)),
        };

        self.expect_operator("=")?;
        let value = self.parse_expression()?;  // now stops before semicolon
        self.expect_delim(';')?;               // correctly consumes the semicolon

        Ok(Statement::VarDecl { name, value })
    }

    fn parse_return(&mut self) -> Result<Statement, String> {
        self.expect_keyword("return")?;
        let value = self.parse_expression()?;  // stops before semicolon
        self.expect_delim(';')?;               // consumes the ';'
        Ok(Statement::Return(value))
    }

}
//parse statements end

//parse expressions start
impl<'a> Parser<'a> {
    fn parse_expression(&mut self) -> Result<Expression, String> {
        let mut left = match self.advance() {
            Some(Token::Literal(LiteralType::Integer(n))) => Expression::Integer(*n),
            Some(Token::Literal(LiteralType::Boolean(b))) => Expression::Boolean(*b),
            Some(Token::Literal(LiteralType::String(s))) => Expression::String(s.clone()),
            Some(Token::Identifier(s)) => Expression::Ident(s.clone()),

            // 👇 handle grouped expressions like (x + y)
            Some(Token::Delimiter('(')) => {
                let expr = self.parse_expression()?; // parse inside the parens
                self.expect_delim(')')?;             // require closing ')'
                expr
            }

            other => return Err(format!("Unexpected token in expression: {:?}", other)),
        };

        // handle binary operators
        while let Some(tok) = self.peek() {
            match tok {
                Token::Operator(op) => {
                    let op_str = op.clone();
                    self.advance(); // consume operator
                    let right = self.parse_expression()?; // parse right side
                    left = Expression::BinaryOp {
                        left: Box::new(left),
                        op: op_str,
                        right: Box::new(right),
                    };
                }
                Token::Delimiter(';') | Token::Delimiter('}') | Token::Delimiter(')') => break,
                _ => break,
            }
        }

        Ok(left)
    }


}
//parse expression end


//parse ignore whitespace start
impl<'a> Parser<'a> {
    fn advance(&mut self) -> Option<&Token> {
        while let Some(tok) = self.tokens.get(self.current) {
            self.current += 1;
            if matches!(tok, Token::Whitespace | Token::Comment) {
                continue;
            }
            return Some(tok);
        }
        None
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens[self.current..]
            .iter()
            .find(|tok| !matches!(tok, Token::Whitespace | Token::Comment))
    }

    fn expect_delim(&mut self, ch: char) -> Result<(), String> {
        while let Some(tok) = self.tokens.get(self.current) {
            match tok {
                Token::Whitespace | Token::Comment => { self.current += 1; continue; }
                Token::Delimiter(c) if *c == ch => { self.current += 1; return Ok(()); }
                other => return Err(format!("Expected delimiter '{}', found {:?}", ch, other)),
            }
        }
        Err(format!("Expected delimiter '{}', found end of input", ch))
    }

}
//parse ignore whitespace end
