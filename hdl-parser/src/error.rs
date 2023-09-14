use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

#[derive(Error,Debug,Diagnostic)]
#[error("HDL Error")]
pub struct HdlError{
    #[source_code]
    src:NamedSource,
    error_pos: SourceSpan 
}