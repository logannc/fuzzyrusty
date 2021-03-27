//! Normalizer trait and default implementations.
//!
//! Normalization is how strings are normalized into characters you want to
//! consider equal. For example, if you want a case insensitive comparison,
//! you might consider the [LowerCaseNormalizer].
//!
//! ```
//! # use fuzzywuzzy::normalization::{Normalizer, LowerCaseNormalizer, FormCNormalizer, ComposedNormalizer};
//! assert_eq!(LowerCaseNormalizer.normalize("this STRING"), LowerCaseNormalizer.normalize("THIS string"));
//! let a1 = "ä"; // U+00E4
//! let a2 = "ä"; // U+0061 + U+0308
//! let a3 = "Ä"; // U+0041 + U+0308
//! assert_ne!(a1, a2);
//! assert_eq!(FormCNormalizer.normalize(a2), a1);
//! let multiple_normalizers = ComposedNormalizer::with(
//!     vec![Box::new(LowerCaseNormalizer), Box::new(FormCNormalizer)]);
//! assert_eq!(multiple_normalizers.normalize(a3), a1);
//! ```

/// Represents a strategy for normalizing string characters into a canonical
/// value of their equivalence class.
///
/// i.e., in a case-insensitive context, 'a' might be the canonical value for
/// the equivalence class of ASCII A's: `['a', 'A']`.
///
/// In addition to implementers of the trait, functions with a matching type
/// signature also work. ```
/// # use fuzzywuzzy::normalization::{Normalizer, LowerCaseNormalizer};
/// let test_string = "test STRING";
/// fn custom_normalizer(s: &str) -> String { s.to_lowercase() }
/// assert_eq!(
///    LowerCaseNormalizer.normalize(&test_string),
///    custom_normalizer.normalize(&test_string));
/// ```
pub trait Normalizer {
    fn normalize(&self, s: &str) -> String;
}

impl<F: Fn(&str) -> String> Normalizer for F {
    fn normalize(&self, s: &str) -> String {
        self(s)
    }
}

/// Doesn't modify any characters. The Identity-transform [Normalizer].
///
/// ```
/// # use fuzzywuzzy::normalization::{Normalizer, PassthroughNormalizer};
/// use rand::{thread_rng, Rng};
/// use rand::distributions::Alphanumeric;
/// let random_string: String = thread_rng()
///              .sample_iter(&Alphanumeric)
///              .take(16).map(char::from).collect();
/// assert_eq!(PassthroughNormalizer.normalize(&random_string), random_string);
/// ```
pub struct PassthroughNormalizer;

impl Normalizer for PassthroughNormalizer {
    fn normalize(&self, s: &str) -> String {
        s.into()
    }
}

/// Compose a sequence of [Normalizer]s together into one [Normalizer].
///
/// They are executed in sequential order.
/// ```
/// # use fuzzywuzzy::normalization::{Normalizer, LowerCaseNormalizer, FormCNormalizer, ComposedNormalizer};
/// let a1 = "ä"; // U+00E4
/// let a2 = "Ä"; // U+0041 + U+0308
/// let multiple_normalizers = ComposedNormalizer::with(
///     vec![Box::new(LowerCaseNormalizer), Box::new(FormCNormalizer)]);
/// assert_eq!(multiple_normalizers.normalize(a2), a1);
/// ```
pub struct ComposedNormalizer {
    normalizers: Vec<Box<dyn Normalizer>>,
}

impl ComposedNormalizer {
    pub fn with(normalizers: Vec<Box<dyn Normalizer>>) -> ComposedNormalizer {
        ComposedNormalizer { normalizers }
    }
}

impl Normalizer for ComposedNormalizer {
    fn normalize(&self, s: &str) -> String {
        let mut current = s.to_owned();
        for normalizer in self.normalizers.iter() {
            current = normalizer.normalize(&current);
        }
        current
    }
}

/// Normalizes strings by lower-casing all letters.
///
/// ```
/// # use fuzzywuzzy::normalization::{Normalizer, LowerCaseNormalizer};
/// assert_eq!(LowerCaseNormalizer.normalize("this STRING"), LowerCaseNormalizer.normalize("THIS string"));
/// ```
pub struct LowerCaseNormalizer;

impl Normalizer for LowerCaseNormalizer {
    fn normalize(&self, s: &str) -> String {
        s.to_lowercase()
    }
}

/// Removes non-ASCII codepoints.
///
/// Notably, this does not ASCII-ify non-ASCII characters, it just removes them.
/// If you want to turn characters into ASCII decompositions, look at
/// [FormDNormalizer], [FormKDNormalizer], or [UnicodeToAsciiNormalizer].
/// ```
/// # use fuzzywuzzy::normalization::{Normalizer, AsciiOnlyNormalizer};
/// assert_eq!(AsciiOnlyNormalizer.normalize("äbc"), "bc");
/// ```
pub struct AsciiOnlyNormalizer;

impl Normalizer for AsciiOnlyNormalizer {
    fn normalize(&self, s: &str) -> String {
        s.chars().filter(char::is_ascii).collect()
    }
}

/// Replaces sequences of characters that are not letters or numbers with a
/// single space.
///
/// Note, for compatibility with Python's fuzzywuzzy which internally uses the
/// `\W` regex character class, we include underscore (`'_'`) as a
/// letter/number.
///
/// There might be other unknown differences between Python's `re` module's
/// implementation of `\W` and Rust's implementation of [char::is_alphanumeric].
/// ```
/// # use fuzzywuzzy::normalization::{Normalizer, SplittingAlphanumericNormalizer};
/// assert_eq!(SplittingAlphanumericNormalizer.normalize("abc   123"), "abc   123");
/// assert_eq!(SplittingAlphanumericNormalizer.normalize("abc!!!123"), "abc   123");
/// assert_eq!(SplittingAlphanumericNormalizer.normalize("   abc123"), "   abc123");
/// // Some codepoints like common diacritics are removed.
/// assert_eq!(SplittingAlphanumericNormalizer.normalize("a\u{0308}bc"), "a bc");
/// // But single-character codepoints like U+00E4 are not.
/// assert_eq!(SplittingAlphanumericNormalizer.normalize("äbc"), "äbc");
/// // Known incompatibility: Python's fuzzywuzzy converts the combining characters below,
/// // but Rust considers them to be alphabetic characters so they are not.
/// // Future versions of fuzzywuzzy-rs may add a new normalizer to handle this.
/// // assert_eq!(FutureCompatibilityNormalizer.normalize("abcØØØकिमपि"), "abcØØØक मप ");
/// assert_eq!(SplittingAlphanumericNormalizer.normalize("abcØØØकिमपि"), "abcØØØकिमपि");
/// ```
pub struct SplittingAlphanumericNormalizer;

// Take care when editing this as
// `replace_non_letters_non_numbers_with_whitespace` delegates to this.
impl Normalizer for SplittingAlphanumericNormalizer {
    fn normalize(&self, s: &str) -> String {
        s.split(|c: char| !(c == '_' || c.is_alphanumeric()))
            .intersperse(" ")
            .collect()
    }
}

// TODO: document
pub struct WhitespaceSplitSortedTokenNormalizer;

impl Normalizer for WhitespaceSplitSortedTokenNormalizer {
    fn normalize(&self, s: &str) -> String {
        let mut v: Vec<_> = s.split_whitespace().collect();
        v.sort_unstable();
        v.join(" ")
    }
}

#[cfg(feature = "segmentation")]
pub use self::unicode_normalizers::*;

#[cfg(feature = "segmentation")]
mod unicode_normalizers {
    use super::Normalizer;
    use unicode_normalization::UnicodeNormalization;

    /// Performs Unicode Normalization Form C (canonical decomposition followed
    /// by canonical composition). Requires default feature "normalization".
    ///
    /// This just delegates to
    /// [unicode_normalization::UnicodeNormalization::nfc].
    ///
    /// ```
    /// # use fuzzywuzzy::normalization::{Normalizer, FormCNormalizer};
    /// let a1 = "ä"; // U+00E4
    /// let a2 = "ä"; // U+0061 + U+0308
    /// assert_ne!(a1, a2);
    /// assert_eq!(FormCNormalizer.normalize(a2), a1);
    /// ```
    pub struct FormCNormalizer;

    impl Normalizer for FormCNormalizer {
        fn normalize(&self, s: &str) -> String {
            s.nfc().collect()
        }
    }

    /// Performs Unicode Normalization Form KC (compatibility decomposition
    /// followed by canonical composition). Requires default feature
    /// "normalization".
    ///
    /// This just delegates to
    /// [unicode_normalization::UnicodeNormalization::nfkc].
    ///
    /// ```
    /// # use fuzzywuzzy::normalization::{Normalizer, FormKCNormalizer};
    /// let a1 = "ä"; // U+00E4
    /// let a2 = "ä"; // U+0061 + U+0308
    /// assert_ne!(a1, a2);
    /// assert_eq!(FormKCNormalizer.normalize(a2), a1);
    /// ```
    pub struct FormKCNormalizer;

    impl Normalizer for FormKCNormalizer {
        fn normalize(&self, s: &str) -> String {
            s.nfkc().collect()
        }
    }

    /// Performs Unicode Normalization Form D (canonical decomposition).
    /// Requires default feature "normalization".
    ///
    /// This just delegates to
    /// [unicode_normalization::UnicodeNormalization::nfd].
    ///
    /// ```
    /// # use fuzzywuzzy::normalization::{Normalizer, FormDNormalizer};
    /// // FormDNormalizer.normalize(U+00E4) == (U+0061 + U+0308)
    /// assert_eq!(FormDNormalizer.normalize("ä"), "a\u{0308}");
    /// ```
    pub struct FormDNormalizer;
    impl Normalizer for FormDNormalizer {
        fn normalize(&self, s: &str) -> String {
            s.nfd().collect()
        }
    }

    /// Performs Unicode Normalization Form KD (compatibility decomposition).
    /// Requires default feature "normalization".
    ///
    /// This just delegates to
    /// [unicode_normalization::UnicodeNormalization::nfkd].
    ///
    /// ```
    /// # use fuzzywuzzy::normalization::{Normalizer, FormKDNormalizer};
    /// // FormKDNormalizer.normalize(U+00E4) == (U+0061 + U+0308)
    /// assert_eq!(FormKDNormalizer.normalize("ä"), "a\u{0308}");
    /// ```
    pub struct FormKDNormalizer;
    impl Normalizer for FormKDNormalizer {
        fn normalize(&self, s: &str) -> String {
            s.nfkd().collect()
        }
    }

    /// Performs CJK Compatibility Ideograph-to-Standarized Variation Sequence
    /// normalization. Requires default feature "normalization".
    ///
    /// This just delegates to
    /// [unicode_normalization::UnicodeNormalization::cjk_compat_variants].
    ///
    /// > "This is not part of the canonical or compatibility decomposition
    /// algorithms, but performing it before those algorithms produces
    /// normalized output which better preserves the intent of the original
    /// text." -- [unicode_normalization](unicode_normalization::
    /// UnicodeNormalization::cjk_compat_variants)
    pub struct CJKNormalizer;
    impl Normalizer for CJKNormalizer {
        fn normalize(&self, s: &str) -> String {
            s.cjk_compat_variants().collect()
        }
    }

    /// Decomposes a string, then removes non-ascii code points. Requires
    /// default feature "normalization".
    ///
    /// Caution is needed when applying this [Normalizer].
    /// While it may improve Latin-script based language comparisons because
    /// they can often decompose largely into ASCII + diacritics,
    /// it will perform poorly on less ASCII-centric languages.
    ///
    /// ```
    /// # use fuzzywuzzy::normalization::{Normalizer, UnicodeToAsciiNormalizer};
    /// // U+00E4
    /// assert_eq!(UnicodeToAsciiNormalizer.normalize("äbc"), "abc");
    /// // U+0061 + U+0308
    /// assert_eq!(UnicodeToAsciiNormalizer.normalize("a\u{0308}bc"), "abc");
    /// // This is probably not what you want!
    /// assert_eq!(UnicodeToAsciiNormalizer.normalize("किमप"), "");
    /// ```
    pub struct UnicodeToAsciiNormalizer;
    impl Normalizer for UnicodeToAsciiNormalizer {
        fn normalize(&self, s: &str) -> String {
            s.nfd().filter(char::is_ascii).collect()
        }
    }
}
