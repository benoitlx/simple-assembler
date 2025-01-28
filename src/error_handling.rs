use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Unrecognized token")]
#[diagnostic(
    code(tokenizer::no_matching_pattern),
    url("https://my-incredible-doc.fr"),
    help("TODO provide the closest pattern")
)]
pub struct UnrecognizedToken {
    #[source_code]
    pub src: NamedSource<String>,

    #[label("This doesn't match any Token pattern")]
    pub src_span: SourceSpan,
}

#[derive(Error, Diagnostic, Debug)]
pub enum TokenizerError {
    #[error(transparent)]
    #[diagnostic(code(tokenizer::io_error), help("try this filename:"))]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    TokenError(#[from] UnrecognizedToken),
}
