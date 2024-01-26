use crate::Error;
use super::DataSource;

/// Loads bank holidays from a URL in JSON format.
/// Uses the `reqwest` client.
pub struct Reqwest<'a> {
    url: &'a str,
}

impl Default for Reqwest<'static> {
    #[inline]
    fn default() -> Self {
        Reqwest::new(crate::SOURCE_URL)
    }
}

impl<'a> Reqwest<'a> {
    /// Create a new `reqwest`-based client for loading bank holidays in JSON format for the given URL.
    #[inline]
    pub fn new(url: &'a str) -> Self {
        Self { url }
    }

    /// Load bank holidays from URL.
    pub async fn load_data_source(&self) -> Result<DataSource, Error> {
        tracing::debug!("Loading bank holidays from {}", self.url);
        reqwest::get(self.url)
            .await?
            .json::<DataSource>()
            .await
            .map(|mut data_source| {
                data_source.sort();
                data_source.add_missing_divisions();
                data_source
            })
            .map_err(Error::from)
    }
}
