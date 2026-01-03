use thiserror::Error;

use crate::tokenization::SourcePosition;

#[derive(Error, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum CssParsingError {
    #[error("incomplete simple block at {0}")]
    IncompleteSimpleBlock(SourcePosition),

    #[error("incomplete function at {0}")]
    IncompleteFunction(SourcePosition),

    #[error("incomplete at-rule at {0}")]
    IncompleteAtRule(SourcePosition),

    #[error("incomplete qualified rule at {0}")]
    IncompleteQualifiedRule(SourcePosition),

    #[error("unexpected end of file in simple block at {0}")]
    EofInSimpleBlock(SourcePosition),

    #[error("unexpected end of file in function at {0}")]
    EofInFunction(SourcePosition),

    #[error("unexpected end of file in at-rule at {0}")]
    EofInAtRule(SourcePosition),

    #[error("unexpected end of file in qualified rule at {0}")]
    EofInQualifiedRule(SourcePosition),

    #[error("unexpected end of file in declaration at {0}")]
    EofInDeclaration(SourcePosition),

    #[error("invalid declaration start at {0}")]
    InvalidDeclarationStart(SourcePosition),

    #[error("invalid declaration name at {0}")]
    InvalidDeclarationName(SourcePosition),

    #[error("missing colon in declaration at {0}")]
    MissingColonInDeclaration(SourcePosition),
}
