//! This module represents an effort to be result-compatible with Python's [fuzzywuzzy](https://github.com/seatgeek/fuzzywuzzy).
//!
//! This module is great for getting started or porting from using [fuzzywuzzy](https://github.com/seatgeek/fuzzywuzzy).
//! The rest of this crate might be more accurate or offer more advanced use
//! cases if your project needs grow beyond that.
//!
//! This module's implementation may change at any time if it improves compliance with [fuzzywuzzy](https://github.com/seatgeek/fuzzywuzzy)'s results.
//!
//! Warning/Note: Almost everything in this module assumes [Codepoint Segmentation](CodePointSegmenter) which is not always appropriate.

pub mod fuzz;
pub mod process;
pub mod string_processing;
pub mod utils;
