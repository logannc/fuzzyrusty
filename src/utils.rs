//! Standalone functions used by the rest of the crate. You might also find them
//! useful.

/// some common short circuiting for ratio finding functions.
/// If the strings are equal, they have a ratio of 100%.
/// If only one of the strings is empty, they have a ratio of 0%.
macro_rules! check_trivial {
    ($s1:expr, $s2:expr) => {
        use core::convert::TryInto;
        if $s1 == $s2 {
            return 100u8.try_into().unwrap();
        }
        if $s1.is_empty() ^ $s2.is_empty() {
            return 0u8.try_into().unwrap();
        }
    };
}
