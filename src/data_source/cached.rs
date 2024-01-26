use super::DataSource;

/// Built-in list of bank holidays used as a backup, in testing or when network requests are not available.
pub struct Cached;

impl Cached {
    /// Create [`DataSource`] from cached data.
    /// Does not need asynchronous loading.
    pub fn cached_data_source() -> DataSource {
        const CACHED_DATA: &[u8] = include_bytes!("bank-holidays.json");
        DataSource::try_from_json(CACHED_DATA)
            .expect("cached data should be valid")
    }
}
