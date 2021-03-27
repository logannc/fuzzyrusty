#![deny(warnings)]
//! Fuzzy string matching like
//! [FuzzyWuzzy.py](https://github.com/seatgeek/fuzzywuzzy) like a boss. It uses
//! [Levenshtein Distance](https://en.wikipedia.org/wiki/Levenshtein_distance)
//! to calculate the differences between sequences in a simple-to-use package.

#![feature(iter_intersperse)]
#[macro_use]
pub mod utils;
pub mod fuzz;
pub mod fuzzywuzzy_compatible;
pub mod normalization;
pub mod primitives;
pub mod segmentation;
