use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::Date;

/// Details of a bank holiday.
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct BankHoliday {
    /// Date of this bank holiday.
    pub date: Date,
    /// Title of this bank holiday.
    pub title: String,
    /// Notes such as “Substitute day”; typically blank.
    pub notes: String,
    /// Bunting (???)
    pub bunting: bool,
}

impl BankHoliday {
    /// New bank holiday with blank notes and no bunting.
    #[inline]
    pub fn new(date: Date, title: String) -> Self {
        Self::new_with_notes(date, title, "".to_owned())
    }

    /// New bank holiday with notes and no bunting.
    #[inline]
    pub fn new_with_notes(date: Date, title: String, notes: String) -> Self {
        BankHoliday {
            date,
            title,
            notes,
            bunting: false,
        }
    }
}

impl AsRef<Date> for BankHoliday {
    #[inline]
    fn as_ref(&self) -> &Date {
        &self.date
    }
}

impl PartialOrd for BankHoliday {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BankHoliday {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
            .then_with(|| self.title.cmp(&other.title))
    }
}

impl fmt::Debug for BankHoliday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.date, self.title)?;
        if !self.notes.is_empty() {
            write!(f, " ({})", self.notes)?;
        }
        Ok(())
    }
}
