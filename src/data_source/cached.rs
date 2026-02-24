use std::marker::PhantomData;

use crate::{Error, PlainDate};
use crate::data_source::{DataSource, LoadDataSource};

/// Built-in list of bank holidays used as a backup, in testing or when network requests are not available.
pub struct Cached<Date: PlainDate>(PhantomData<fn() -> Date>);

impl<Date: PlainDate> Default for Cached<Date> {
    #[inline(always)]
    fn default() -> Self {
        Cached(PhantomData)
    }
}

impl<Date: PlainDate> Cached<Date> {
    /// Create [`DataSource`] from cached data.
    /// Does not need asynchronous loading.
    pub fn cached_data_source(&self) -> DataSource<Date> {
        const CACHED_DATA: &[u8] = include_bytes!("bank-holidays.json");
        DataSource::try_from_json(CACHED_DATA)
            .expect("cached data should be valid")
    }
}

impl<Date: PlainDate> LoadDataSource<Date> for Cached<Date> {
    #[inline]
    async fn load_data_source(&self) -> Result<DataSource<Date>, Error> {
        Ok(self.cached_data_source())
    }
}
