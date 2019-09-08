use derive_more::*;

use std::error::Error;

/// SGF parsing, or traversal, related errors
#[derive(Debug, Display)]
#[display(fmt = "{}", kind)]
pub struct SgfError {
    pub kind: SgfErrorKind,
    source: Option<Box<dyn Error + Send + Sync + 'static>>,
}

/// Describes what kind of error we're dealing with
#[derive(Debug, Display, Eq, PartialEq)]
pub enum SgfErrorKind {
    #[display(fmt = "Error parsing SGF file")]
    ParseError,
    #[display(fmt = "Variation not found")]
    VariationNotFound,
    #[display(fmt = "Root token found in a non root node")]
    InvalidRootTokenPlacement,
}

impl Error for SgfError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|boxed| boxed.as_ref() as &(dyn Error + 'static))
    }
}

impl From<SgfErrorKind> for SgfError {
    fn from(kind: SgfErrorKind) -> SgfError {
        SgfError { kind, source: None }
    }
}

impl SgfError {
    pub fn parse_error(err: impl Error + Send + Sync + 'static) -> Self {
        SgfError {
            kind: SgfErrorKind::ParseError,
            source: Some(Box::new(err)),
        }
    }

    pub fn variation_not_found(err: impl Error + Send + Sync + 'static) -> Self {
        SgfError {
            kind: SgfErrorKind::VariationNotFound,
            source: Some(Box::new(err)),
        }
    }

    pub fn invalid_root_token_placment(err: impl Error + Send + Sync + 'static) -> Self {
        SgfError {
            kind: SgfErrorKind::InvalidRootTokenPlacement,
            source: Some(Box::new(err)),
        }
    }
}
