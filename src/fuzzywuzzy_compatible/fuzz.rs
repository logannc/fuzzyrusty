//! Implementations guaranteed for fuzzywuzzy compatibility.
//!
//! Looking at the implementations of these methods may be informative for using
//! the rest of the library.
// TODO: link fuzzywuzzy.

use crate::fuzz::{ratio, ratio_full};
use crate::fuzzywuzzy_compatible::utils;
use crate::normalization::{
    AsciiOnlyNormalizer, ComposedNormalizer, LowerCaseNormalizer, Normalizer,
    PassthroughNormalizer, SplittingAlphanumericNormalizer, WhitespaceSplitSortedTokenNormalizer,
};
use crate::primitives::{get_matching_blocks, simple_ratio, Score};
use crate::segmentation::{CodePointSegmenter, Segmenter};
// TODO: replace with no_std hashbrown
use std::collections::HashSet;

/// Return the ratio of the most similar substring as a number between 0 and
/// 100.
///
/// The most similar substring is determined by finding the "optimal" alignment
/// of the two strings. Then the similarity ratio of the aligned strings is
/// returned.
///
/// Note: in compatibility with fuzzywuzzy, a suboptimal sequence alignment
/// algorithm is used. In future versions, this may change.
///
/// ```
/// # use fuzzywuzzy::fuzz::partial_ratio;
/// assert_eq!(partial_ratio("", ""), 100);
/// assert_eq!(partial_ratio("", "nonempty"), 0);
/// assert_eq!(partial_ratio("ab", "abcd"), 100);
/// assert_eq!(partial_ratio("bc", "abcd"), 100);
/// assert_eq!(partial_ratio("cd", "abcd"), 100);
/// assert_eq!(partial_ratio("ad", "abcd"), 50);
/// assert_eq!(partial_ratio("ac", "abcd"), 50);
/// assert_eq!(partial_ratio("hello", "hello world"), 100);
/// assert_eq!(partial_ratio("new york mets", "the new york mets"), 100);
/// assert_eq!(partial_ratio("the new york mets", "new york mets"), 100);
/// // Note the order dependence due to not finding the optimal alignment
/// assert_eq!(partial_ratio(
///    "what about supercalifragilisticexpialidocious",
///    "supercalifragilisticexpialidocious about what"), 76);
/// assert_eq!(partial_ratio(
///    "supercalifragilisticexpialidocious about what",
///    "what about supercalifragilisticexpialidocious"), 86);
/// ```
pub fn partial_ratio(a: &str, b: &str) -> Score {
    partial_ratio_full(a, b, PassthroughNormalizer, CodePointSegmenter)
}

// TODO: document
// TODO: reimplement with proper optimal alignment in crate::fuzz (don't change
// this one)
pub fn partial_ratio_full<N, S>(a: &str, b: &str, normalizer: N, segmenter: S) -> Score
where
    N: Normalizer,
    S: for<'inner> Segmenter<'inner>,
{
    check_trivial!(a, b);
    let normalized_a = normalizer.normalize(a);
    let normalized_b = normalizer.normalize(b);
    let segmented_a = segmenter.segment(&normalized_a);
    let segmented_b = segmenter.segment(&normalized_b);
    let (shorter, longer) = if segmented_a.len() <= segmented_b.len() {
        (segmented_a, segmented_b)
    } else {
        (segmented_b, segmented_a)
    };
    // This is a suboptimal alignment algorithm that will be replaced in a new,
    // non-fuzzywuzzy compatible function.
    let blocks = get_matching_blocks(&shorter, &longer);
    let mut max = 0.try_into().unwrap();
    for (i, j, _) in blocks {
        let long_start = if j > i { j - i } else { 0 };
        let long_end = std::cmp::min(long_start + shorter.len(), longer.len());
        let long_substr = &longer[long_start..long_end];
        let r = simple_ratio(&shorter, long_substr).try_into().unwrap();
        if r > 99 {
            return 100.try_into().unwrap();
        } else if r > max {
            max = r;
        }
    }
    max
}

/// Return a measure of the sequences' similarity between 0 and 100, but sort
/// the token before comparing.
///
/// This essentially sorts 'words' in each string and then runs `ratio`.
///
/// By default, force_ascii and full_process should be true.
///
/// ```
/// # use fuzzywuzzy::fuzzywuzzy_compatible::fuzz::token_sort_ratio;
/// assert_eq!(token_sort_ratio("hello world", "world hello", true, true), 100);
/// assert_eq!(token_sort_ratio("new york mets", "the new york mets", true, true), 87);
/// # // This passes because the WhitespaceSplitSortedTokenNormalizer uses `split_whitespace` which implicitly trims
/// assert_eq!(token_sort_ratio("    new york mets", "the new york mets   ", true, true), 87);
/// assert_eq!(token_sort_ratio("new york mets", "new YORK mets", true, true), 100);
/// assert_eq!(token_sort_ratio(
///    "what about supercalifragilisticexpialidocious",
///    "supercalifragilisticexpialidocious about what", true, true), 100);
/// assert_eq!(token_sort_ratio(
///    "what about supercalifragilisticexpialidociousx",
///    "supercalifragilisticexpialidociousy about what", true, true), 98);
/// ```
pub fn token_sort_ratio(a: &str, b: &str, force_ascii: bool, full_process: bool) -> Score {
    match (force_ascii, full_process) {
        (false, true) => ratio_full(
            a,
            b,
            ComposedNormalizer::with(vec![
                Box::new(SplittingAlphanumericNormalizer),
                Box::new(LowerCaseNormalizer),
                Box::new(WhitespaceSplitSortedTokenNormalizer),
            ]),
            CodePointSegmenter,
        ),
        (true, true) => ratio_full(
            a,
            b,
            ComposedNormalizer::with(vec![
                Box::new(AsciiOnlyNormalizer),
                Box::new(SplittingAlphanumericNormalizer),
                Box::new(LowerCaseNormalizer),
                Box::new(WhitespaceSplitSortedTokenNormalizer),
            ]),
            CodePointSegmenter,
        ),
        (_, false) => ratio_full(a, b, PassthroughNormalizer, CodePointSegmenter),
    }
}

/// Return the ratio of the most similar substring as a number between 0 and
/// 100, but sort the tokens before comparing.
///
/// This essentially sorts 'words' in each string and then runs `partial_ratio`.
///
/// By default, force_ascii and full_process should be true.
///
/// ```
/// # use fuzzywuzzy::fuzzywuzzy_compatible::fuzz::partial_token_sort_ratio;
/// assert_eq!(partial_token_sort_ratio("hello world", "world hello", true, true), 100);
/// assert_eq!(partial_token_sort_ratio("new york mets", "the new york mets", true, true), 69);
/// assert_eq!(partial_token_sort_ratio("new york mets", "new YORK mets", true, true), 100);
/// assert_eq!(partial_token_sort_ratio("new york mets vs atlanta braves", "atlanta braves vs new york mets", true, true), 100);
/// assert_eq!(partial_token_sort_ratio(
///    "what about supercalifragilisticexpialidocious",
///    "supercalifragilisticexpialidocious about what", true, true), 100);
/// assert_eq!(partial_token_sort_ratio(
///    "what about supercalifragilisticexpialidociousx",
///    "supercalifragilisticexpialidociousy about what", true, true), 98);
/// ```
pub fn partial_token_sort_ratio(a: &str, b: &str, force_ascii: bool, full_process: bool) -> Score {
    match (force_ascii, full_process) {
        (false, true) => partial_ratio_full(
            a,
            b,
            ComposedNormalizer::with(vec![
                Box::new(SplittingAlphanumericNormalizer),
                Box::new(LowerCaseNormalizer),
                Box::new(WhitespaceSplitSortedTokenNormalizer),
            ]),
            CodePointSegmenter,
        ),
        (true, true) => partial_ratio_full(
            a,
            b,
            ComposedNormalizer::with(vec![
                Box::new(AsciiOnlyNormalizer),
                Box::new(SplittingAlphanumericNormalizer),
                Box::new(LowerCaseNormalizer),
                Box::new(WhitespaceSplitSortedTokenNormalizer),
            ]),
            CodePointSegmenter,
        ),
        (_, false) => partial_ratio_full(a, b, PassthroughNormalizer, CodePointSegmenter),
    }
}

/// Quick `ratio` comparison between two strings.
pub fn qratio(a: &str, b: &str, force_ascii: bool) -> Score {
    if force_ascii {
        ratio_full(
            a,
            b,
            ComposedNormalizer::with(vec![
                Box::new(AsciiOnlyNormalizer),
                Box::new(SplittingAlphanumericNormalizer),
                Box::new(LowerCaseNormalizer),
                Box::new(WhitespaceSplitSortedTokenNormalizer),
            ]),
            CodePointSegmenter,
        )
    } else {
        ratio_full(
            a,
            b,
            ComposedNormalizer::with(vec![
                Box::new(SplittingAlphanumericNormalizer),
                Box::new(LowerCaseNormalizer),
                Box::new(WhitespaceSplitSortedTokenNormalizer),
            ]),
            CodePointSegmenter,
        )
    }
}

/// micro-quick-ratio: `qratio` comparison between two strings without forcing
/// to ascii.
pub fn uqratio(s1: &str, s2: &str) -> Score {
    qratio(s1, s2, false)
}

////////////////////////////////////////////////////////////////////////////////////
// TODO: The following methods can be converted to use Normalizers but there is
// no urgency to do so because they do funky things that mix the 'state' between
// the comparison strings.
////////////////////////////////////////////////////////////////////////////////////

/// Find all alphanumeric tokens in each string...
///  # treat them as a set
///  # construct two strings of the form:
/// <sorted_intersection><sorted_remainder>  # take ratios of those two strings
///  # controls for unordered partial matches
fn token_set(s1: &str, s2: &str, partial: bool, force_ascii: bool, full_process: bool) -> u8 {
    check_trivial!(s1, s2);
    let (p1, p2) = if full_process {
        (
            utils::full_process(s1, force_ascii),
            utils::full_process(s2, force_ascii),
        )
    } else {
        (s1.to_string(), s2.to_string())
    };
    let t1: HashSet<_> = p1.split_whitespace().collect();
    let t2: HashSet<_> = p2.split_whitespace().collect();
    let mut intersection: Vec<_> = t1.intersection(&t2).cloned().collect();
    let mut diff1to2: Vec<_> = t1.difference(&t2).cloned().collect();
    let mut diff2to1: Vec<_> = t2.difference(&t1).cloned().collect();
    intersection.sort_unstable();
    diff1to2.sort_unstable();
    diff2to1.sort_unstable();
    let intersect_str = intersection.join(" ");
    let diff1to2_str = diff1to2.join(" ");
    let diff2to1_str = diff2to1.join(" ");
    let combined_1to2 = vec![intersect_str.to_string(), diff1to2_str.to_string()];
    let combined_1to2 = combined_1to2
        .into_iter()
        .filter(|s| !s.is_empty())
        .intersperse(" ".into())
        .collect::<String>();
    let combined_2to1 = vec![intersect_str.to_string(), diff2to1_str.to_string()];
    let combined_2to1 = combined_2to1
        .into_iter()
        .filter(|s| !s.is_empty())
        .intersperse(" ".into())
        .collect::<String>();
    if partial {
        *vec![
            partial_ratio(&intersect_str, &combined_1to2).score(),
            partial_ratio(&intersect_str, &combined_2to1).score(),
            partial_ratio(&combined_1to2, &combined_2to1).score(),
        ]
        .iter()
        .max()
        .unwrap()
    } else {
        dbg!(&intersect_str, &combined_1to2, &combined_2to1);
        *vec![
            ratio(&intersect_str, &combined_1to2).score(),
            ratio(&intersect_str, &combined_2to1).score(),
            ratio(&combined_1to2, &combined_2to1).score(),
        ]
        .iter()
        .max()
        .unwrap()
    }
}

/// Return the ratio of the most similar substring constructed from the strings
/// treated as sets, as a number between 0 and 100.
///
/// Creates three sets from the two strings:
///  1. a sorted set containing words in both strings, joined by spaces
///  2. a sorted set containing words in both strings followed by words only in
/// the first string, joined by spaces  3. a sorted set containing words in both
/// strings followed by words only in the second string, joined by spaces
///
/// Note: the set of words in the intersection and difference are sorted before
/// being concatenated and joined by spaces.
///
/// The ratio is computed between all three, pairwise, and the maximum is
/// returned.
///
/// ```
/// # use fuzzywuzzy::fuzzywuzzy_compatible::fuzz::token_set_ratio;
/// assert_eq!(token_set_ratio("hello world", "world hello", true, true), 100);
/// assert_eq!(token_set_ratio("new york mets", "the new york mets", true, true), 100);
/// assert_eq!(token_set_ratio("new york mets", "new YORK mets", true, true), 100);
/// assert_eq!(token_set_ratio("new york mets vs atlanta braves", "atlanta braves vs new york mets", true, true), 100);
/// assert_eq!(token_set_ratio(
///    "what about supercalifragilisticexpialidocious",
///    "supercalifragilisticexpialidocious about what", true, true), 100);
/// ```
pub fn token_set_ratio(s1: &str, s2: &str, force_ascii: bool, full_process: bool) -> u8 {
    // trivial check omitted because this is a shallow delegator to token_set which
    // checks.
    token_set(s1, s2, false, force_ascii, full_process)
}

/// Return the partial ratio of the most similar substring constructed from the
/// strings treated as sets, as a number between 0 and 100.
///
/// Creates three sets from the two strings:
///  1. a sorted set containing words in both strings, joined by spaces
///  2. a sorted set containing words in both strings followed by words only in
/// the first string, joined by spaces  3. a sorted set containing words in both
/// strings followed by words only in the second string, joined by spaces
///
/// Note: the set of words in the intersection and difference are sorted before
/// being concatenated and joined by spaces.
///
/// The partial ratio is computed between all three, pairwise, and the maximum
/// is returned.
///
/// ```
/// # use fuzzywuzzy::fuzzywuzzy_compatible::fuzz::partial_token_set_ratio;
/// assert_eq!(partial_token_set_ratio("hello world", "world hello", true, true), 100);
/// assert_eq!(partial_token_set_ratio("new york mets", "the new york mets", true, true), 100);
/// assert_eq!(partial_token_set_ratio("new york mets", "new YORK mets", true, true), 100);
/// assert_eq!(partial_token_set_ratio("new york mets - atlanta braves", "atlanta braves - new york city mets", true, true), 100);
/// assert_eq!(partial_token_set_ratio(
///    "what about supercalifragilisticexpialidocious",
///    "supercalifragilisticexpialidocious about what", true, true), 100);
/// ```
pub fn partial_token_set_ratio(s1: &str, s2: &str, force_ascii: bool, full_process: bool) -> u8 {
    // trivial check omitted because this is a shallow delegator to token_set which
    // checks.
    token_set(s1, s2, true, force_ascii, full_process)
}

/// Return a measure of the sequences' similarity between 0 and 100, using a
/// composite algorithm.
///
/// *Steps in the order they occur*
///  1. Run full_process from utils on both strings
///  2. Short circuit if this makes either string empty
///  3. Take the ratio of the two processed strings (`fuzz::ratio`)
///  4. Run checks to compare the length of the strings
///     * If one of the strings is more than 1.5 times as long as the other use
///       partial_ratio comparisons - scale partial results by 0.9 (this makes
///       sure only full results can return 100)
///     * If one of the strings is over 8 times as long as the other instead
///       scale by 0.6
///  5. Run the other ratio functions
///     * if using partial ratio functions call partial_ratio,
///       partial_token_sort_ratio and partial_token_set_ratio scale all of
///       these by the ratio based on length
///     * otherwise call token_sort_ratio and token_set_ratio
///     * all token based comparisons are scaled by 0.95 (on top of any partial
///       scalars)
///  6. Take the highest value from these results
///     round it and return it as an integer.
///
/// TODO: function is hard-coded to use partial functions?
///
/// ```
/// # use fuzzywuzzy::fuzzywuzzy_compatible::fuzz::wratio;
/// assert_eq!(wratio("", "", true, true), 100);
/// assert_eq!(wratio("foo", "bar", true, true), 0);
/// assert_eq!(wratio("hello world", "hello world", true, true), 100);
/// assert_eq!(wratio("hello world", "world hello", true, true), 95);
/// assert_eq!(wratio("new york mets", "new YORK mets", true, true), 100);
/// assert_eq!(wratio("new york mets", "the wonderful new york mets", true, true), 90);
/// assert_eq!(wratio("new york mets vs atlanta braves", "atlanta braves vs new york mets", true, true), 95);
/// ```
pub fn wratio(s1: &str, s2: &str, force_ascii: bool, full_process: bool) -> u8 {
    check_trivial!(s1, s2);
    let (p1, p2) = if full_process {
        (
            utils::full_process(s1, force_ascii),
            utils::full_process(s2, force_ascii),
        )
    } else {
        (s1.to_string(), s2.to_string())
    };
    let (p1r, p2r) = (p1.as_str(), p2.as_str());
    let mut try_partial = true;
    const UNBASE_SCALE: f64 = 0.95;
    let mut partial_scale = 0.90;

    let base = ratio(p1r, p2r).score();
    let (p1_len, p2_len) = (p1.chars().count(), p2.chars().count());
    let len_ratio = std::cmp::max(p1_len, p2_len) as f64 / std::cmp::min(p1_len, p2_len) as f64;

    // if strings are similar length, don't use partials
    if len_ratio < 1.5 {
        try_partial = false;
    }

    // if one string is much shorter than the other
    if len_ratio > 8.0 {
        partial_scale = 0.6;
    }

    if try_partial {
        let partial = partial_ratio(p1r, p2r).score() as f64 * partial_scale;
        let ptsor = partial_token_sort_ratio(p1r, p2r, true, false).score() as f64
            * UNBASE_SCALE
            * partial_scale;
        let ptser =
            partial_token_set_ratio(p1r, p2r, true, false) as f64 * UNBASE_SCALE * partial_scale;
        // This conversion to u8 from the maximum f64 seems spooky, but let's hope
        // nothing bad happens!
        return vec![base as f64, partial, ptsor, ptser]
            .iter()
            .cloned()
            .fold(f64::NAN, f64::max)
            .round() as u8;
    }
    let tsor = token_sort_ratio(p1r, p2r, true, false).score() as f64 * UNBASE_SCALE;
    let tser = token_set_ratio(p1r, p2r, true, false) as f64 * UNBASE_SCALE;
    vec![base as f64, tsor, tser]
        .iter()
        .cloned()
        .fold(f64::NAN, f64::max)
        .round() as u8
}

/// Runs `wratio` without forcing to ascii.
pub fn uwratio(s1: &str, s2: &str, full_process: bool) -> u8 {
    // trivial check omitted because this is a shallow delegator to wratio which
    // checks.
    wratio(s1, s2, false, full_process)
}
