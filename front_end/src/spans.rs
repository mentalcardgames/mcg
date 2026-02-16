use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OwnedSpan {
    pub start: usize,
    pub end: usize,
    pub start_pos: (usize, usize),
    pub end_pos: (usize, usize),
}

pub struct OwnedPosition {

}

impl From<pest::Span<'_>> for OwnedSpan {
    fn from(input: pest::Span) -> Self {
        Self {
            start: input.start(),
            end: input.end(),
            start_pos: input.start_pos().line_col(),
            end_pos: input.end_pos().line_col()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(bound = "T: Serialize + DeserializeOwned")] // Tell Serde how to handle the generic
pub struct Spanned<T> {
    pub node: T,
    pub span: OwnedSpan,
}
