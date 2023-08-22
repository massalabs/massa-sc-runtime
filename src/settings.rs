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

// Same value as in Massa repo (MAX_FUNCTION_NAME_LENGTH)
pub(crate) const MAX_FUNCTION_NAME_LENGTH: u16 = u16::MAX;

// Same value as in Massa repo (MAX_PARAMETERS_SIZE)
pub(crate) const MAX_PARAMETERS_SIZE: u32 = 10_000_000;
