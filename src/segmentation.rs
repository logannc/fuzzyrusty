//! Segmenter trait and default implementations.
//!
//! Segmentation is how strings are split into tokens for comparison.
//! For example, two strings that *visually appear* identical might have
//! different byte-level representations.
//!
//! Take `ä` and `ä`. Visually, these should be identical. However, the former
//! is Unicode character [ä (U+00E4)](https://www.compart.com/en/unicode/U+00E4)
//! while the latter is two adjacent Unicode characters [a (U+0061)](https://www.compart.com/en/unicode/U+0061)
//! and [◌̈ (U+0308)](https://www.compart.com/en/unicode/U+0308).
//!
//! Depending on the [Segmenter] used, comparing these strings will return
//! different results.
//!
//! ```
//! # use fuzzywuzzy::segmentation::{Segmenter, ByteSegmenter};
//! // U+00E4
//! assert_eq!(ByteSegmenter.segment("ä"), vec![0xc3u8, 0xa4u8]);
//! // U+0061 + U+0308
//! assert_eq!(ByteSegmenter.segment("ä"), vec![0x61u8, 0xccu8, 0x88u8]);
//! ```
//! Given this segmentation, we would expect a comparison to return 0%
//! similarity because every byte is different!
//!
//! However, even with more advanced segmentation strategies like [code point
//! segmentation](CodePointSegmenter) or [grapheme
//! segmentation](GraphemeSegmenter), these will still have 0% similarity by any
//! comparison algorithm.
//!
//! ```
//! # use fuzzywuzzy::segmentation::{Segmenter, CodePointSegmenter, GraphemeSegmenter};
//! // U+00E4
//! assert_eq!(CodePointSegmenter.segment("ä"), vec!['ä']);
//! assert_eq!(GraphemeSegmenter.segment("ä"), vec!["ä"]);
//! // U+0061 + U+0308
//! assert_eq!(CodePointSegmenter.segment("ä"), vec!['a', '\u{0308}']);
//! assert_eq!(GraphemeSegmenter.segment("ä"), vec!["ä"]);
//! // U+00E4 vs. U+0061 + U+0308
//! assert_ne!(GraphemeSegmenter.segment("ä"), GraphemeSegmenter.segment("ä"));
//! ```
//!
//! In order to usefully compare strings like these,
//! [normalization][crate::normalization] must be done prior to segmentation.

/// Represents a strategy for segmenting a string into units for comparison.
///
/// The trait is also implemented for functions matching the signature of the
/// `segment` method.
///
/// In addition to implementers of the trait, functions with a matching type
/// signature also work.
///
/// ```
/// # use fuzzywuzzy::segmentation::{Segmenter, CodePointSegmenter};
/// let test_string = "test STRING";
/// fn custom_segmenter(s: &str) -> Vec<char> { s.chars().collect() }
/// assert_eq!(
///    CodePointSegmenter.segment(&test_string),
///    custom_segmenter.segment(&test_string));
/// ```
pub trait Segmenter<'a> {
    /// The type of the unit of comparison this strategy operates on.
    type Output: 'a + Eq;
    /// Produces units of comparison from a string according to the segmentation
    /// strategy.
    fn segment(&self, s: &'a str) -> Vec<Self::Output>;
}

impl<'a, F: Fn(&str) -> Vec<T>, T: 'a + Eq> Segmenter<'a> for F {
    type Output = T;
    fn segment(&self, s: &'a str) -> Vec<Self::Output> {
        self(s)
    }
}

/// A strategy for segmenting strings into their constituent bytes.
///
/// ```
/// # use fuzzywuzzy::segmentation::{Segmenter, ByteSegmenter};
/// // U+00E4
/// assert_eq!(ByteSegmenter.segment("ä"), vec![0xc3u8, 0xa4u8]);
/// // U+0061 + U+0308
/// assert_eq!(ByteSegmenter.segment("ä"), vec![0x61u8, 0xccu8, 0x88u8]);
/// ```
pub struct ByteSegmenter;

impl<'a> Segmenter<'a> for ByteSegmenter {
    // Returns an owned `Vec<u8>` because allocating additional `u8`s is cheaper
    // than pointers into the original string.
    type Output = u8;
    fn segment(&self, s: &'a str) -> Vec<Self::Output> {
        s.as_bytes().iter().copied().collect()
    }
}

/// A strategy for segmenting strings into their constituent Unicode code
/// points.
///
/// Internally, this is just `s.chars().collect()`.
///
/// Note that `char` is a Unicode Scalar Value which is a subset of Unicode code
/// points disallowing surrogates. UTF-8, which all Rust strings are guaranteed
/// to be, also disallows surrogates. So all of the Unicode Scalar Values
/// produced here are UTF-8 code points.
///
/// ```
/// # use fuzzywuzzy::segmentation::{Segmenter, CodePointSegmenter};
/// // U+00E4
/// assert_eq!(CodePointSegmenter.segment("ä"), vec!['ä']);
/// // U+0061 + U+0308
/// assert_eq!(CodePointSegmenter.segment("ä"), vec!['a', '\u{0308}']);
/// // 'किमपि' (kimapi) and 'किमप' (kimapa)
/// assert_eq!(CodePointSegmenter.segment("किमपि"), vec!['क', 'ि', 'म', 'प', 'ि']);
/// assert_eq!(CodePointSegmenter.segment("किमप"), vec!['क', 'ि', 'म', 'प']);
/// ```
pub struct CodePointSegmenter;

impl<'a> Segmenter<'a> for CodePointSegmenter {
    type Output = char;
    fn segment(&self, s: &'a str) -> Vec<Self::Output> {
        s.chars().collect()
    }
}

// TODO: document
pub struct WhitespaceSegmenter;

impl<'a> Segmenter<'a> for WhitespaceSegmenter {
    type Output = &'a str;
    fn segment(&self, s: &'a str) -> Vec<Self::Output> {
        s.split_whitespace().collect()
    }
}

// TODO: document
pub struct SortedWhitespaceSegmenter;

impl<'a> Segmenter<'a> for SortedWhitespaceSegmenter {
    type Output = &'a str;
    fn segment(&self, s: &'a str) -> Vec<Self::Output> {
        let mut v: Vec<_> = s.split_whitespace().collect();
        v.sort_unstable();
        v
    }
}

#[cfg(feature = "segmentation")]
pub use self::unicode_segmenters::*;

#[cfg(feature = "segmentation")]
mod unicode_segmenters {
    use super::Segmenter;
    use unicode_segmentation::UnicodeSegmentation;

    /// A strategy for segmenting strings into their constituent Unicode
    /// graphemes. Requires default feature "segmentation".
    ///
    /// This just delegates to [unicode_segmentation].
    ///
    /// ```
    /// # use fuzzywuzzy::segmentation::{Segmenter, GraphemeSegmenter};
    /// // U+00E4
    /// assert_eq!(GraphemeSegmenter.segment("ä"), vec!["ä"]);
    /// // U+0061 + U+0308
    /// assert_eq!(GraphemeSegmenter.segment("ä"), vec!["ä"]);
    /// // 'किमपि' (kimapi) and 'किमप' (kimapa)
    /// assert_eq!(GraphemeSegmenter.segment("किमपि"), vec!["कि", "म", "पि"]);
    /// assert_eq!(GraphemeSegmenter.segment("किमप"), vec!["कि", "म", "प"]);
    /// ```
    pub struct GraphemeSegmenter;

    impl<'a> Segmenter<'a> for GraphemeSegmenter {
        type Output = &'a str;
        fn segment(&self, s: &'a str) -> Vec<Self::Output> {
            s.graphemes(true).collect()
        }
    }

    // TODO: document
    pub struct UnicodeWordSegmenter;

    impl<'a> Segmenter<'a> for UnicodeWordSegmenter {
        type Output = &'a str;
        fn segment(&self, s: &'a str) -> Vec<Self::Output> {
            s.split_word_bounds().collect()
        }
    }
}
