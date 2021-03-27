//! Convenience methods to process fuzzy matching queries for common use cases.
//!
//! Lack of functions compared to [fuzzywuzzy](https://github.com/seatgeek/fuzzywuzzy) are just 'get top N', at this point.
use crate::fuzzywuzzy_compatible::fuzz::wratio;
use crate::fuzzywuzzy_compatible::utils::full_process;
use crate::primitives::{Match, Processor, Score, Scorer, Sorter};
use core::cmp::Ordering;
use core::convert::TryInto;

/// The default scorer used for functions in this module. Delegates to [wratio].
///
/// The `&&str` is a consequence of the type signature of the [Scorer] trait
/// and Rust disliking `str` without a reference.
pub fn default_scorer(query: &&str, choice: String) -> Score {
    wratio(query, &choice, true, true).try_into().unwrap()
}

/// The default processor used for functions in this module. Delegates to
/// [full_process].
///
/// The `&&str` is a consequence of many functions generically accepting `A` and
/// passing in `&A` to [Processor]s. and Rust disliking `str` without a
/// reference.
pub fn default_processor(s: &&str) -> String {
    full_process(*s, false)
}

/// Quickly compare and return scored [Match]es of the choices against the
/// query. Delegates to [extract_without_order_full].
///
/// ```
/// # use core::convert::TryInto;
/// # use fuzzywuzzy::primitives::{Match, Score};
/// # use fuzzywuzzy::fuzzywuzzy_compatible::process::extract_without_order;
/// let query = "bar";
/// let choices = vec!["foo", "bar", "baz"];
/// assert_eq!(extract_without_order(query, &choices), vec![Match{ item: "foo", score: 0.try_into().unwrap() }, Match{ item: "bar", score: 100.try_into().unwrap() }, Match{ item: "baz", score: 67.try_into().unwrap() }, ]);
/// ```
pub fn extract_without_order<'a>(query: &'a str, choices: &[&'a str]) -> Vec<Match<&'a str>> {
    extract_without_order_full(
        query,
        choices,
        default_processor,
        default_scorer,
        Score::new(0).unwrap(),
    )
    .into_iter()
    .map(|s| Match {
        item: *s.item,
        score: s.score,
    })
    .collect()
}

// TODO: add to a doctest here....
// let choices = vec![
//     "new york mets vs chicago cubs",
//     "chicago cubs vs chicago white sox",
//     "philladelphia phillies vs atlanta braves",
//     "braves vs mets",
// ];
// let expected_results = vec![
//    ("new york mets vs chicago cubs".to_string(), 86u8),
//    ("chicago cubs vs chicago white sox".to_string(), 86u8),
//    ("philladelphia phillies vs atlanta braves".to_string(), 54u8),
//    ("braves vs mets".to_string(), 57u8)
// ];
// assert_eq!(
//     extract_without_order(
//         "brave new cubs",
//         choices,
//         |s, b| s.into(), // an alternative to full_process.
//         &wratio,
//         0),
//     expected_results);
// ```

/// Return scored [Match]es of the choices against the query with your choice of
/// [Processor] and [Scorer].
///
/// ```
/// # use core::convert::TryInto;
/// # use fuzzywuzzy::primitives::{Match, Score};
/// # use fuzzywuzzy::fuzzywuzzy_compatible::process::{ default_processor, default_scorer, extract_without_order_full };
/// let query = "bar";
/// let choices = vec!["foo", "bar", "baz"];
/// assert_eq!(extract_without_order_full(query, &choices, default_processor, default_scorer, 0.try_into().unwrap()), vec![Match{ item: &"foo", score: 0.try_into().unwrap() }, Match{ item: &"bar", score: 100.try_into().unwrap() }, Match{ item: &"baz", score: 67.try_into().unwrap() }, ]);
/// ```
pub fn extract_without_order_full<'a, 'b, A: 'a, B, C, P, S>(
    query: A,
    choices: &'b [B],
    processor: P,
    scorer: S,
    score_cutoff: Score,
) -> Vec<Match<&'b B>>
where
    P: Processor<&'b B, C>,
    // because 'inner can be *any* lifetime (including very short ones), this tells Rust that the
    // scorer doesn't need our &query for long
    S: for<'inner> Scorer<&'inner A, C>,
{
    if choices.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    for c in choices.iter() {
        let score = scorer.score(&query, processor.process(c));
        if score >= score_cutoff {
            result.push(Match { item: c, score });
        }
    }
    result
}

/// Quickly, stably dedupe strings by fuzzily comparing them to each other.
/// Delegates to [dedupe_full].
pub fn dedupe<'a>(items: &[&'a str], threshold: Score) -> Vec<&'a str> {
    let scorer = |a: &&str, b: &&str| wratio(a, b, true, true).try_into().unwrap();
    let sorter = |a: &&str, b: &&str| match a.len().cmp(&b.len()) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => a.cmp(b),
    };
    dedupe_full(items, threshold, scorer, sorter, true)
        .into_iter()
        .map(|s| *s)
        .collect()
}

/// Given a list of items, fuzzily deduplicate them by comparing them to each
/// other using the [Scorer].
///
/// For each item in the list, we compare it to all other items. All items with
/// a [Score] exceeding `threshold` are collected and stably sorted according to
/// the [Sorter] which is emitted as the canonical representation.
///
/// After each item is replaced by the representative for its batch, the
/// intermediate result containing all canonical values are sorted (using the
/// natural sorting order for the type) and deduped with a window of size 2.
///
/// If the `stable` parameter is true, the result will be sorted in the order
/// they first appear in the input.
///
/// There can be strange chains of logic with hidden or unexpected results if
/// the scorer is not commutative (e.g., `score(a,b) != score(b,a)`).
/// For example, given [A, B, C] where A ~ B, B > A, B ~ C, C > B, we might
/// return [B, C] because B took A's place and C took B's place.
///
/// ```
/// # use core::cmp::Ordering;
/// # use core::convert::TryInto;
/// # use fuzzywuzzy::primitives::{Match, Score};
/// # use fuzzywuzzy::fuzzywuzzy_compatible::fuzz::wratio;
/// # use fuzzywuzzy::fuzzywuzzy_compatible::process::{ default_processor, default_scorer, dedupe_full };
/// let frodo_baggin = "Frodo Baggin";
/// let frodo_baggins = "Frodo Baggins";
/// let f_baggins = "F. Baggins";
/// let samwise = "Samwise G.";
/// let gandalf = "Gandalf";
/// let bilbo = "Bilbo Baggins";
/// let contains_dupes = vec![frodo_baggin, samwise, frodo_baggins, samwise, gandalf, bilbo, f_baggins];
/// // Notice that later instances of the same 'group' are gone but the order of the groups is maintained.
/// let expected_stable = vec![&frodo_baggins, &samwise, &gandalf, &bilbo];
/// // ... but not when we don't require `stable`.
/// let expected_unstable = vec![&samwise, &gandalf, &frodo_baggins, &bilbo];
/// let scorer = |a: &&str, b: &&str| wratio(a, b, true, true).try_into().unwrap();
/// let sorter = |a: &&str, b: &&str| {
///     match a.len().cmp(&b.len()) {
///         Ordering::Less => Ordering::Less,
///         Ordering::Greater => Ordering::Greater,
///         Ordering::Equal => a.cmp(b),
///     }
/// };
/// assert_eq!(dedupe_full(&contains_dupes, 70.try_into().unwrap(), scorer, sorter, true), expected_stable);
/// assert_eq!(dedupe_full(&contains_dupes, 70.try_into().unwrap(), scorer, sorter, false), expected_unstable);
/// ```
pub fn dedupe_full<'a, A: 'a + Eq + Ord>(
    items: &'a [A],
    threshold: Score,
    scorer: impl Scorer<&'a A, &'a A>,
    sorter: impl Sorter<&'a A>,
    stable: bool,
) -> Vec<&'a A> {
    let mut extractor = Vec::new();
    for item in items.iter() {
        let mut matches = extract_without_order_full(
            item,
            items,
            |a: &'a A| a,
            |a: &&'a A, b: &'a A| scorer.score(a, b),
            threshold,
        );
        matches.sort_by(|a, b| sorter.sort(a.item, b.item).reverse());
        extractor.extend(matches.iter().map(|m| m.item).take(1));
    }
    // unstable case first because it is easier
    if !stable {
        // unstably sort with our order
        extractor.sort_unstable_by(|a, b| a.cmp(b).reverse());
        extractor.dedup();
        extractor
    } else {
        // to maintain the order we had before deletion:
        // 1. we save the index
        // 2. sort by our items,
        // 3. dedup
        // 4. re-sort by our original indices

        // (1) - save/enumerate our original indices
        let mut sorted: Vec<(usize, &A)> = extractor.into_iter().enumerate().collect();
        // (2) - sort by our items
        sorted.sort_unstable_by(|(_, a), (_, b)| a.cmp(b).reverse());
        let sorted_length = sorted.len();
        // (3) - dedup
        let mut deduped =
            sorted
                .into_iter()
                .fold(Vec::with_capacity(sorted_length), |mut v, (index, item)| {
                    match v.last() {
                        Some(&(_, last)) => {
                            if last != item {
                                v.push((index, item));
                            }
                        }
                        None => v.push((index, item)),
                    }
                    v
                });
        // (4) - re-sort by our indices
        // unstable sort is okay here because the indices are unique
        deduped.sort_unstable_by(|(a1, _), (b1, _)| a1.cmp(b1));
        deduped.into_iter().map(|(_, item)| item).collect()
    }
}
