//! Ported functions from fuzzywuzzy.string_processing

use crate::normalization::{Normalizer, SplittingAlphanumericNormalizer};

/// Replaces sequences of characters that are not letters or numbers with a
/// single space.
///
/// Note that this function does not take into account
/// [normalization](crate::normalization) or
/// [segmentation](crate::segmentation).
///
/// Note, for compatibility with Python's fuzzywuzzy which internally uses the
/// `\W` regex character class, we include underscore (`'_'`) as a
/// letter/number.
///
/// There might be other unknown differences between Python's `re` module's
/// implementation of `\W` and Rust's implementation of [char::is_alphanumeric].
/// ```
/// # use fuzzywuzzy::fuzzywuzzy_compatible::string_processing::replace_non_letters_non_numbers_with_whitespace;
/// assert_eq!(replace_non_letters_non_numbers_with_whitespace("abc   123"), "abc   123");
/// assert_eq!(replace_non_letters_non_numbers_with_whitespace("abc!!!123"), "abc   123");
/// // Some codepoints like common diacritics are removed.
/// assert_eq!(replace_non_letters_non_numbers_with_whitespace("a\u{0308}bc"), "a bc");
/// // But single-character codepoints like U+00E4 are not.
/// assert_eq!(replace_non_letters_non_numbers_with_whitespace("äbc"), "äbc");
/// // Known incompatibility: Python's fuzzywuzzy converts the combining characters below,
/// // but Rust considers them to be alphabetic characters so they are not.
/// // Future versions of fuzzywuzzy-rs may fix this.
/// // assert_eq!(replace_non_letters_non_numbers_with_whitespace("abcØØØकिमपि"), "abcØØØक मप ");
/// assert_eq!(replace_non_letters_non_numbers_with_whitespace("abcØØØकिमपि"), "abcØØØकिमपि");
/// ```
pub fn replace_non_letters_non_numbers_with_whitespace(s: &str) -> String {
    SplittingAlphanumericNormalizer.normalize(s)
}
