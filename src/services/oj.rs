//! hack file for Online Judge

#[macro_export]
macro_rules! oj_output {
    ($($arg:tt)*) => {
        // the oj checkes stderr
        eprintln!($($arg)*);
    };
}