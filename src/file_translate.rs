use std::fs;
use std::io;

//get file contents gets what is inside and read_file translates it so tokenize can use it
fn get_file_contents<'a, E>(file_path: &str, buffer: &'a mut String) -> Result<&'a str, E>
where
    E: From<io::Error>,
{
    *buffer = fs::read_to_string(file_path)?;
    Ok(buffer.as_str())
}

//read_file just reads the file and puts it in a way so that tokenize can use the result
pub fn read_file(buffer: &mut String) -> Result<&str, std::io::Error> {
    get_file_contents("myfile.txt", buffer)
}

