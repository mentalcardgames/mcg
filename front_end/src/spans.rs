#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub node: T,
    pub span: OwnedSpan,
}