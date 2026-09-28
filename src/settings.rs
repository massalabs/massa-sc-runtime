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
