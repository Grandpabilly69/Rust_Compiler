mod lex_layer;
mod file_translate;
mod syntax_analyzer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use lex_layer::Token::*;
    use lex_layer::LiteralType::*;

    let mut buffer = std::string::String::new();

    let contents = file_translate::read_file(&mut buffer)?;  // now `?` works

    let tokens = lex_layer::tokenize::<std::io::Error>(Ok(contents))?;

    let mut parser = syntax_analyzer::Parser::new(&tokens);
    match parser.parse_function() {
        Ok(func) => println!("{:#?}", func),
        Err(e) => eprintln!("Parse error: {}", e),
    }

    Ok(())
}

fn check_tokens() -> Result<(), std::io::Error> {
    let mut buffer = String::new();
    let contents = file_translate::read_file(&mut buffer)?;           // Result<&str, io::Error>
    let tokens = lex_layer::tokenize::<std::io::Error>(Ok(contents))?; // tokenize consumes Result<&str, E>

    //prints the consumed tokens correctly based on file created
    println!("Tokens: {:?}", tokens);
    Ok(())
}