use crate::lang::Catalog;

/// Temporary compatibility boundary for proven V1 helpers that still call
/// `lang::catalog()`. V2 owns the catalog explicitly; only calls made through
/// this adapter receive the thread-local binding.
pub(super) fn with_catalog<T>(catalog: &'static Catalog, operation: impl FnOnce() -> T) -> T {
    let _binding = crate::lang::bind_catalog(catalog);
    operation()
}
