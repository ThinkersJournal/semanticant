use semanticant_core::Verdict;

/// A deterministic similarity or relatedness measure between two things of type `T`.
/// Higher means more similar. The verdict carries the reasons for the score.
pub trait Measure<T: ?Sized> {
    /// Score `a` against `b`.
    fn score(&self, a: &T, b: &T) -> Verdict<f64>;
}
