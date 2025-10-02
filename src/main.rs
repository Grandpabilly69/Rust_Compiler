mod lex_layer;
mod file_translate;


fn main() -> Result<(), std::io::Error> {
    let mut buffer = String::new();
    let contents = file_translate::read_file(&mut buffer)?;           // Result<&str, io::Error>
    let tokens = lex_layer::tokenize::<std::io::Error>(Ok(contents))?; // tokenize consumes Result<&str, E>

    //prints the consumed tokens correctly based on file created
    println!("Tokens: {:?}", tokens);
    Ok(())
}
