cfg_if::cfg_if! {
    if #[cfg(unix)] {
        pub mod unix;
        pub use unix::*;
    }
}