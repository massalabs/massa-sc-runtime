/// Execution component version introduced by massa MIP-0002 (`MipComponent::Execution` v2).
/// Kept as a literal here to avoid a massa-versioning dependency; it must stay equal to
/// massa's `MIP_0002_EXECUTION_VERSION`.
///
/// From this version on, wasmv1 modules are no longer executed and the paginated
/// datastore-key imports are resolved. Before it, an updated node keeps the previous
/// behavior: wasmv1 modules still run, and the new imports are absent, so instantiation
/// fails exactly as on a non-updated node. A host that cannot report its version is
/// treated as pre-activation.
pub const MIP_0002_EXECUTION_VERSION: u32 = 2;

/// Maximum number of datastore keys one paginated call may return.
/// The host must apply the same bound.
pub const MAX_DATASTORE_KEYS_PAGE: u32 = 500;

pub(crate) const MAIN: &str = "main";

pub(crate) fn max_number_of_pages() -> u32 {
    64
}

pub(crate) fn max_datastore_entry_count() -> usize {
    100_000
}

pub(crate) fn max_op_datastore_entry_count() -> usize {
    128
}
