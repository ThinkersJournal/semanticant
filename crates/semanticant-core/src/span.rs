use crate::CoreError;

/// A half-open byte range `start..end` into the ORIGINAL text, so a caller can quote evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// Build a span; `start > end` is an error.
    pub fn new(start: usize, end: usize) -> Result<Self, CoreError> {
        if start > end {
            return Err(CoreError::InvalidSpan { start, end });
        }
        Ok(Self { start, end })
    }
    /// First byte (inclusive).
    pub fn start(&self) -> usize {
        self.start
    }
    /// Last byte (exclusive).
    pub fn end(&self) -> usize {
        self.end
    }
    /// Length in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    /// True for a zero-length span.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
    /// True if `other` lies wholly inside `self`.
    pub fn contains(&self, other: &Span) -> bool {
        self.start <= other.start && other.end <= self.end
    }
    /// True if the two spans share at least one byte.
    pub fn overlaps(&self, other: &Span) -> bool {
        self.start < other.end && other.start < self.end
    }
    /// The smallest span covering both.
    pub fn union(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
    /// The quoted text, or `None` if the span is out of range or not on char boundaries.
    pub fn slice<'a>(&self, text: &'a str) -> Option<&'a str> {
        text.get(self.start..self.end)
    }
}
