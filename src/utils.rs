use std::num::NonZeroUsize;

/// Find the number of cores on the current device, or return a default of `8`.
pub fn get_core_count() -> i64 {
    std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(8).unwrap())
        .get()
        .try_into()
        .expect("Cannot convert number of CPUs")
}
