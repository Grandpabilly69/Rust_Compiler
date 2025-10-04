use std::cmp::PartialEq;

fn create_vec() -> Vec<Token> {
    let input_code: Vec<Token> = Vec::new();
    return input_code;
}
//the above creates a vector to use

//this is for all the types of tokens there can be in the language
#[derive(Debug, PartialEq)]
pub enum Token{
    Keyword(String), // e.g., func, if, for, var, return
    Identifier(String), // e.g., "my_variable", "function_name"
    Literal(LiteralType), // e.g., numbers, strings, booleans
    Operator(String), // e.g., "+", "-", "="
    Delimiter(char), // e.g., "(", "{", ";"
    Whitespace,
    Comment,
    Unknown(char),
}

//This is for the different types of variables there can be
#[derive(Debug, PartialEq)]
pub enum LiteralType {
    Integer(i64),
    Boolean(bool),
    String(String),
}



//uses tokens and categorizes them
//input and is_whitespace is giving issues.
pub fn tokenize<E>(input: Result<&str, E>) -> Result<Vec<Token>, E> {
    let s = input?; // if Err(E), return it immediately
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            _ if c.is_whitespace() => {
                chars.next();
                if !tokens.last().map_or(false, |t| t == &Token::Whitespace) {
                    tokens.push(Token::Whitespace);
                }
            }
            '/' if chars.clone().nth(1) == Some('/') => {
                while let Some(ch) = chars.next() {
                    if ch == '\n' {
                        break;
                    }
                }
                tokens.push(Token::Comment);
            }
            '+' | '-' | '*' | '/' | '=' => {
                tokens.push(Token::Operator(c.to_string()));
                chars.next();
            }
            '(' | ')' | '{' | '}' | ';' => {
                tokens.push(Token::Delimiter(c));
                chars.next();
            }
            _ if c.is_alphabetic() || c == '_' => {
                let mut ident_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                match ident_str.as_str() {
                    "func" | "var" | "if" | "else" | "return" => tokens.push(Token::Keyword(ident_str)),
                    "truth" => tokens.push(Token::Literal(LiteralType::Boolean(true))),
                    "falsy" => tokens.push(Token::Literal(LiteralType::Boolean(false))),
                    _ => tokens.push(Token::Identifier(ident_str)),
                }
            }
            _ if c.is_ascii_digit() => {
                let mut num_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if let Ok(num) = num_str.parse::<i64>() {
                    tokens.push(Token::Literal(LiteralType::Integer(num)));
                } else {
                    tokens.push(Token::Unknown(c));
                }
            }
            '"' => {
                chars.next();
                let mut string_content = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        break;
                    }
                    string_content.push(ch);
                }
                tokens.push(Token::Literal(LiteralType::String(string_content)));
            }
            _ => {
                tokens.push(Token::Unknown(c));
                chars.next();
            }
        }
    }

    Ok(tokens)
}
