use std::fs::{read_to_string, File};
use pretty_assertions::assert_eq;

#[path = "../src/lexer.rs"]
mod lexer;

#[test]
fn test_lexer() {
    use lexer::Token;
    use logos::Logos;
    use std::fs::OpenOptions;
    use std::io::{Read, Write};

    let source_file_path = "tests/real_test/realistic_test.asm";
    let temp_file_path = "tests/.temp/realistic_test.token_stream.temp";
    let expected_file_path = "tests/real_test/realistic_test.token_stream";

    let mut temp_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(temp_file_path)
        .unwrap();

    if let Ok(mut source_file) = File::open(source_file_path) {
        let mut content = String::new();
        let _ = source_file.read_to_string(&mut content);

        let mut lex = Token::lexer(content.as_str());

        while let Some(result) = lex.next() {
            writeln!(temp_file, "{:?}", result).unwrap();
        }
    }

    let content1 = read_to_string(expected_file_path).unwrap();
    let content2 = read_to_string(temp_file_path).unwrap();

    assert_eq!(content1, content2);
}
