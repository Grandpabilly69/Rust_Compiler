use std::cmp::PartialEq;

fn create_Vec() -> Vec<Token> {
    let mut input_code: Vec<Token> = Vec::new();
    return input_code;
}
//the above creates a vector to use

//this is for all the types of tokens there can be in the language
enum Token{
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
enum LiteralType {
    Integer(i64),
    String(String),
    Boolean(bool),
    // Add other literal types as needed
}

//not sure what below does might need to edit below
impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

//uses tokens and catagorises them
fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = create_Vec();
    let mut chars = input.chars().peekable(); // peekable is important for being able to see next char without touching it

    while let Some(&c) = chars.peek() {
        match c {
            // Handle whitespace
            _ if c.is_whitespace() => {
                chars.next(); // Consume whitespace
                if !tokens.last().map_or(false, |t| *t == Token::Whitespace) {
                    tokens.push(Token::Whitespace);
                }
                //if block checks if the last token was whitespace to ensure that there are not 2 whitespaces pushed at once
            }
            //the below checks if there are 2 forward slashes together.
            //If there is then it knows that there is a comment
            '/' if chars.clone().nth(1) == Some('/') => {
                while let Some(ch) = chars.next() {
                    if ch == '\n' {
                        break;
                    }
                }
                tokens.push(Token::Comment);
            }
            //The below is for operators
            '+' | '-' | '*' | '/' | '=' => {
                tokens.push(Token::Operator(c.to_string()));
                chars.next();
            }
            //checks delimeters. Basically if it finds opening bracket then it knows there has to be a closing bracket.
            '(' | ')' | '{' | '}' | ';' => {
                tokens.push(Token::Delimiter(c));
                chars.next();
            }
            //Checks and handles delimeters
            _ if c.is_alphabetic() || c == '_' => {
                let mut ident_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                //Checking for Identifiers like stated below
                match ident_str.as_str() {
                    //The names for functions, variable, if then, else
                    "func" | "var" | "if" | "else" => tokens.push(Token::Keyword(ident_str)),
                    //True
                    "truth" => tokens.push(Token::Literal(LiteralType::Boolean(true))),
                    //False
                    "falsy" => tokens.push(Token::Literal(LiteralType::Boolean(false))),
                    _ => tokens.push(Token::Identifier(ident_str)),
                }
            }
            //Handeling digits
            _ if c.is_digit(10) => {
                let mut num_str = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_digit(10) {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                //if this statement is true then it pushes a literal
                if let Ok(num) = num_str.parse::<i64>() {
                    tokens.push(Token::Literal(LiteralType::Integer(num)));
                } else {
                    //handels errors
                    // Handle malformed numbers
                    tokens.push(Token::Unknown(c)); // Or a specific error token
                }
            }
            // Handle string literals (example: double-quoted strings)
            '"' => {
                chars.next(); // Consume opening quote
                let mut string_content = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        break;
                    }
                    string_content.push(ch);
                }
                tokens.push(Token::Literal(LiteralType::String(string_content)));
            }
            //default case/ handels unknown values
            _ => {
                tokens.push(Token::Unknown(c));
                chars.next();
            }
        }
    }

    return tokens;
}