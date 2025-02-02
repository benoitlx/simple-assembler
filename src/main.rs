mod constants;
#[path = "lexer/lexer.rs"]
mod lexer;

use miette::Result;
fn main() -> Result<()> {
    let _ = lexer::lex_from_file("tests/test.asm")?;

    Ok(())
}
