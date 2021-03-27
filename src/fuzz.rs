//! Fuzzy string matching scoring primitives.

use crate::normalization::{Normalizer, PassthroughNormalizer};
use crate::primitives::{simple_ratio, Score};
use crate::segmentation::{CodePointSegmenter, Segmenter};

// Re-export compatibility impls where we don't have better ones yet.
pub use crate::fuzzywuzzy_compatible::fuzz::{partial_ratio, partial_ratio_full};

/// Returns the ratio of the length of matching character sequences to the sum
/// of the length of the input strings as a number between 0 and 100.
///
/// Take, for example, `"cd"` and `"abcd"`.
/// The matching sequence is `"cd"` with a length of 2.
///
/// It is present in both strings, so we count it twice for a total length of
/// matching character sequences of 4.
///
/// The sum of the length of the input strings is `"abcd".len() (4) + "cd".len()
/// (2) = 6`.
///
/// Therefore the returned value is `(4f32/6f32).round() = 67`
///
/// ```
/// # use fuzzywuzzy::fuzz::ratio;
/// assert_eq!(ratio("", "").score(), 100);
/// assert_eq!(ratio("", "nonempty").score(), 0);
/// assert_eq!(ratio("cd", "abcd").score(), 67);
/// assert_eq!(ratio("new york mets", "new york mets").score(), 100);
/// assert_eq!(ratio("new york mets", "new YORK mets").score(), 69);
/// assert_eq!(ratio("hello test", "hello world").score(), 57);
/// ```
pub fn ratio(a: &str, b: &str) -> Score {
    ratio_full(a, b, PassthroughNormalizer, CodePointSegmenter)
}

// TODO: document
pub fn ratio_full<N, S>(a: &str, b: &str, normalizer: N, segmenter: S) -> Score
where
    N: Normalizer,
    S: for<'inner> Segmenter<'inner>,
{
    check_trivial!(a, b);
    let normalized_a = normalizer.normalize(a);
    let normalized_b = normalizer.normalize(b);
    let segmented_a = segmenter.segment(&normalized_a);
    let segmented_b = segmenter.segment(&normalized_b);
    simple_ratio(&segmented_a, &segmented_b).try_into().unwrap()
}

#[cfg(test)]
mod test {
    use super::ratio;
    #[test]
    fn ratio_unicode() {
        let list = [
            ("スマホでchance", "chance", 75),
            ("học", "hoc", 67),
            ("ρɪc", "pic", 33),
            ("quốc", "quoc", 75),
            ("trước", "truoc", 60),
            ("thực", "thuc", 75),
            ("我刚上传了一张照片到facebook", "facebook", 62),
            ("お名前.com", "com", 60),
            ("っˇωˇc", "w", 0),
            ("出会いを探すならpcmax", "pcmax", 56),
            ("化粧cas", "cas", 75),
            ("fòllòwbáck", "followback", 70),
        ];
        for (a, b, r) in list.iter() {
            assert_eq!(ratio(a, b).score(), *r);
        }
    }
}
