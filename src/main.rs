use crate::lex_layer::Token;

mod lex_layer;
mod file_translate;
mod syntax_analyzer;
mod semantic_analyzer;
mod intermediate_code_generator;
mod optimizer;
mod target_code_generator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //allows to use enums from lexer
    use lex_layer::LiteralType::*;

    //creates tokens from lexer to use for syntax analyzer
    let mut buffer = std::string::String::new();
    let contents = file_translate::read_file(&mut buffer)?;
    let tokens = lex_layer::tokenize::<std::io::Error>(Ok(contents))?;

    println!("{:?}", tokens);


    check_sem_syn_ic(tokens);


    Ok(())
}

fn check_sem_syn_ic(tokens: Vec<Token>) {
    let mut parser = syntax_analyzer::Parser::new(&tokens);
    match parser.parse_function() {
        Ok(func) => {
            println!("AST: {:#?}", func);

            let mut sema = semantic_analyzer::SemanticAnalyzer::new();
            match sema.analyze_function(&func) {
                Ok(_) => {
                    println!("Semantic analysis passed");

                    let mut irgen = intermediate_code_generator::IRGenerator::new();
                    let ir = irgen.generate_function(&func);
                    println!("Intermediate Code:\n{:#?}", ir);

                    let optimized = optimizer::optimize_ir(ir.clone());

                    println!("Optimized IR:\n{:#?}", optimized);

                    // after IR generation:
                    let vm_prog = target_code_generator::lower_ir_to_vm(&ir);
                    println!("VM instrs: {:#?}", vm_prog.instrs);

                    let mut vm = target_code_generator::VM::new();
                    let result = vm.run(&vm_prog);
                    println!("Result: {:?}", result);

                }
                Err(e) => eprintln!("Semantic error: {}", e),
            }
        }
        Err(e) => eprintln!("Parse error: {}", e),
    }
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