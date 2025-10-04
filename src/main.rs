mod lex_layer;
mod file_translate;
mod syntax_analyzer;
mod semantic_analyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //allows to use enums from lexer
    use lex_layer::Token::*;
    use lex_layer::LiteralType::*;

    //creates tokens from lexer to use for syntax analyzer
    let mut buffer = std::string::String::new();
    let contents = file_translate::read_file(&mut buffer)?;
    let tokens = lex_layer::tokenize::<std::io::Error>(Ok(contents))?;


    //analyzes syntax of code and prepares tokens into parse tree
    let mut parser = syntax_analyzer::Parser::new(&tokens);
    match parser.parse_function() {
        Ok(func) => println!("{:#?}", func),
        Err(e) => eprintln!("Parse error: {}", e),
    }

    //prepares tokens into parse tree for semantic analyzer
    let mut parser = syntax_analyzer::Parser::new(&tokens);
    match parser.parse_function() {
        Ok(func) => {
            println!("AST: {:#?}", func);

            let mut sema = semantic_analyzer::SemanticAnalyzer::new();
            match sema.analyze_function(&func) {
                Ok(_) => println!("Semantic analysis passed"),
                Err(e) => eprintln!("Semantic error: {}", e),
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }

    Ok(())
}

//this is for error checking by showing the tokens
fn check_tokens() -> Result<(), std::io::Error> {
    let mut buffer = String::new();
    let contents = file_translate::read_file(&mut buffer)?;           // Result<&str, io::Error>
    let tokens = lex_layer::tokenize::<std::io::Error>(Ok(contents))?; // tokenize consumes Result<&str, E>

    //prints the consumed tokens correctly based on file created
    println!("Tokens: {:?}", tokens);
    Ok(())
}