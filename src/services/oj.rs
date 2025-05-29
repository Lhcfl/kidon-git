//! hack file for Online Judge

#[macro_export]
macro_rules! oj_output {
    ($($arg:tt)*) => {
        // the oj checkes stderr
        eprintln!($($arg)*);
    };
}

#[macro_export]
macro_rules! console_output {
    ($($arg:tt)*) => {
        {
            #[cfg(feature = "development")]
            const SHOULD_PRINT : bool = true;
            #[cfg(not(feature = "development"))]
            const SHOULD_PRINT : bool = false;
            
            if SHOULD_PRINT {
                println!($($arg)*);
            }
        }
    };
}