use crate::Span;

/// One step of why a result is what it is. `rule` is a STABLE, documented identifier
/// (a consumer may match on it); `note` is free English text for humans and may change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reason {
    /// Stable rule id, e.g. `"text.sentence.abbreviation"`. Dotted lowercase by convention.
    pub rule: &'static str,
    /// The part of the original text this reason is about, if any.
    pub span: Option<Span>,
    /// Human-readable explanation. Not stable.
    pub note: String,
}

impl Reason {
    /// A reason with no span.
    pub fn new(rule: &'static str, note: impl Into<String>) -> Self {
        Self {
            rule,
            span: None,
            note: note.into(),
        }
    }
    /// A reason about a span of the text.
    pub fn at(rule: &'static str, span: Span, note: impl Into<String>) -> Self {
        Self {
            rule,
            span: Some(span),
            note: note.into(),
        }
    }
}

/// A value together with the reasons for it. Tools return this wherever a decision was made.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict<T> {
    /// The result.
    pub value: T,
    /// Why, in the order the rules fired.
    pub reasons: Vec<Reason>,
}

impl<T> Verdict<T> {
    /// A verdict with no reasons yet.
    pub fn new(value: T) -> Self {
        Self {
            value,
            reasons: Vec::new(),
        }
    }
    /// Add a reason (builder style).
    pub fn because(mut self, reason: Reason) -> Self {
        self.reasons.push(reason);
        self
    }
    /// Transform the value, keeping the reasons.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Verdict<U> {
        Verdict {
            value: f(self.value),
            reasons: self.reasons,
        }
    }
    /// Chain a dependent verdict; reasons accumulate in order.
    pub fn and_then<U>(mut self, f: impl FnOnce(T) -> Verdict<U>) -> Verdict<U> {
        let next = f(self.value);
        self.reasons.extend(next.reasons);
        Verdict {
            value: next.value,
            reasons: self.reasons,
        }
    }
}
