use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnedSpan {
    pub start: usize,
    pub end: usize,
}

impl From<pest::Span<'_>> for OwnedSpan {
    fn from(span: pest::Span) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")] // Tell Serde how to handle the generic
pub struct Spanned<T> {
    pub node: T,
    pub span: OwnedSpan,
}
