//! Misc. helpers as to not clog up the main.rs

// Macro from https://users.rust-lang.org/t/show-value-only-in-debug-mode/43686
/// Wrapper for dbg! that only prints in debug mode.
macro_rules! debug {
    ($($e:expr),+) => {
        {
            #[cfg(debug_assertions)]
            {
                dbg!($($e),+)
            }
            #[cfg(not(debug_assertions))]
            {
                ($($e),+)
            }
        }
    };
}
