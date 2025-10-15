use std::marker::PhantomData;

use crate::{Error, PlainDate};
use crate::data_source::{DataSource, LoadDataSource};

/// Loads bank holidays from a URL in JSON format.
/// Uses the `reqwest` client.
pub struct Reqwest<'a, Date: PlainDate> {
    url: &'a str,
    _phantom: PhantomData<fn() -> Date>,
}

impl<Date: PlainDate> Default for Reqwest<'static, Date> {
    #[inline]
    fn default() -> Self {
        Reqwest::new(crate::SOURCE_URL)
    }
}

impl<'a, Date: PlainDate> Reqwest<'a, Date> {
    /// Create a new `reqwest`-based client for loading bank holidays in JSON format for the given URL.
    #[inline]
    pub const fn new(url: &'a str) -> Self {
        Self { url, _phantom: PhantomData }
    }

    /// Load bank holidays from URL.
    pub async fn load_data_source(&self) -> Result<DataSource<Date>, Error> {
        tracing::debug!("Loading bank holidays from {}", self.url);
        reqwest::get(self.url)
            .await?
            .json::<DataSource<Date>>()
            .await
            .map(|mut data_source| {
                data_source.sort();
                data_source.add_missing_divisions();
                data_source
            })
            .map_err(Error::from)
    }
}

impl<'a, Date: PlainDate> LoadDataSource<Date> for Reqwest<'a, Date> {
    #[inline]
    async fn load_data_source(&self) -> Result<DataSource<Date>, Error> {
        self.load_data_source().await
    }
}
