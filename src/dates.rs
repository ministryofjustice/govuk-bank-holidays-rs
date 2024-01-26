use std::fmt::{Debug, Display, Formatter};

#[cfg(feature = "chrono")]
use chrono::{Datelike, Duration as DurationImpl, NaiveDate as DateImpl};
use serde::{Deserialize, Serialize};
#[cfg(feature = "time")]
use time::{Date as DateImpl, Duration as DurationImpl};

use crate::Error;

#[cfg(all(feature = "chrono", feature = "time"))]
compile_error!("Features `chrono` and `time` cannot be enabled at the same time");

#[cfg(not(any(feature = "chrono", feature = "time")))]
compile_error!("One of `chrono` or `time` features must be enabled");

/// A date without time zone information.
/// Uses `chrono` or `time` crate’s implementation depending on feature flag.
/// Use [`AsRef`] or [`Into`] to get the underlying type.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone, Deserialize, Serialize)]
pub struct Date(DateImpl);

impl From<DateImpl> for Date {
    #[inline]
    fn from(date: DateImpl) -> Self {
        Date(date)
    }
}

impl From<Date> for DateImpl {
    #[inline]
    fn from(date: Date) -> Self {
        date.0
    }
}

impl AsRef<DateImpl> for Date {
    #[inline]
    fn as_ref(&self) -> &DateImpl {
        &self.0
    }
}

impl Display for Date {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// A day of the week from the Gregorian calendar.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[cfg(feature = "chrono")]
impl From<chrono::Weekday> for Weekday {
    fn from(weekday: chrono::Weekday) -> Self {
        match weekday {
            chrono::Weekday::Mon => Weekday::Monday,
            chrono::Weekday::Tue => Weekday::Tuesday,
            chrono::Weekday::Wed => Weekday::Wednesday,
            chrono::Weekday::Thu => Weekday::Thursday,
            chrono::Weekday::Fri => Weekday::Friday,
            chrono::Weekday::Sat => Weekday::Saturday,
            chrono::Weekday::Sun => Weekday::Sunday,
        }
    }
}

#[cfg(feature = "chrono")]
impl From<Weekday> for chrono::Weekday {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Monday => chrono::Weekday::Mon,
            Weekday::Tuesday => chrono::Weekday::Tue,
            Weekday::Wednesday => chrono::Weekday::Wed,
            Weekday::Thursday => chrono::Weekday::Thu,
            Weekday::Friday => chrono::Weekday::Fri,
            Weekday::Saturday => chrono::Weekday::Sat,
            Weekday::Sunday => chrono::Weekday::Sun,
        }
    }
}

#[cfg(feature = "time")]
impl From<time::Weekday> for Weekday {
    fn from(weekday: time::Weekday) -> Self {
        match weekday {
            time::Weekday::Monday => Weekday::Monday,
            time::Weekday::Tuesday => Weekday::Tuesday,
            time::Weekday::Wednesday => Weekday::Wednesday,
            time::Weekday::Thursday => Weekday::Thursday,
            time::Weekday::Friday => Weekday::Friday,
            time::Weekday::Saturday => Weekday::Saturday,
            time::Weekday::Sunday => Weekday::Sunday,
        }
    }
}

#[cfg(feature = "time")]
impl From<Weekday> for time::Weekday {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Monday => time::Weekday::Monday,
            Weekday::Tuesday => time::Weekday::Tuesday,
            Weekday::Wednesday => time::Weekday::Wednesday,
            Weekday::Thursday => time::Weekday::Thursday,
            Weekday::Friday => time::Weekday::Friday,
            Weekday::Saturday => time::Weekday::Saturday,
            Weekday::Sunday => time::Weekday::Sunday,
        }
    }
}

impl Date {
    /// Today’s date.
    #[cfg(feature = "chrono")]
    pub fn today() -> Date {
        chrono::Local::now().date_naive().into()
    }

    /// Today’s date.
    #[cfg(feature = "time")]
    pub fn today() -> Date {
        time::OffsetDateTime::now_local().expect("Cannot get now in local timezone").date().into()
    }

    /// Create a date from day, month and year integers.
    #[cfg(feature = "chrono")]
    pub fn try_from_components(year: i32, month: u32, day: u32) -> Result<Self, Error> {
        match DateImpl::from_ymd_opt(year, month, day) {
            Some(date) => Ok(Date(date)),
            None => Err(Error::InvalidDate),
        }
    }

    /// Create a date from day, month and year integers.
    #[cfg(feature = "time")]
    pub fn try_from_components(year: i32, month: u32, day: u32) -> Result<Self, Error> {
        match DateImpl::from_calendar_date(year, (month as u8).try_into().map_err(|_| Error::InvalidDate)?, day as u8) {
            Ok(date) => Ok(Date(date)),
            Err(_) => Err(Error::InvalidDate),
        }
    }

    /// Get year, month and day for this date.
    pub fn into_components(self) -> (i32, u32, u32) {
        (self.year(), self.month(), self.day())
    }

    /// Get the date’s year.
    #[inline]
    pub fn year(&self) -> i32 {
        self.0.year()
    }

    /// Get the date’s month.
    #[cfg(feature = "chrono")]
    #[inline]
    pub fn month(&self) -> u32 {
        self.0.month()
    }


    /// Get the date’s month.
    #[cfg(feature = "time")]
    #[inline]
    pub fn month(&self) -> u32 {
        u8::from(self.0.month()) as u32
    }

    /// Get the date’s day.
    #[cfg(feature = "chrono")]
    #[inline]
    pub fn day(&self) -> u32 {
        self.0.day()
    }

    /// Get the date’s day.
    #[cfg(feature = "time")]
    #[inline]
    pub fn day(&self) -> u32 {
        self.0.day().into()
    }

    /// Get the date’s day of the week.
    #[inline]
    pub fn weekday(&self) -> Weekday {
        self.0.weekday().into()
    }

    /// Following date.
    pub fn next_day(&self) -> Self {
        Date(self.0 + DurationImpl::days(1))
    }

    /// Preceding date.
    pub fn previous_day(&self) -> Self {
        Date(self.0 - DurationImpl::days(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates() {
        let expected = Date::try_from_components(2023, 2, 3)
            .expect("Date could not be created");
        let parsed: Date = serde_json::from_value(serde_json::Value::String("2023-02-03".to_owned()))
            .expect("Date could not be parsed");
        assert_eq!(parsed, expected);

        let end_of_january = parsed.previous_day().previous_day().previous_day();
        assert_eq!(end_of_january.year(), 2023);
        assert_eq!(end_of_january.month(), 1);
        assert_eq!(end_of_january.day(), 31);
        assert_eq!(end_of_january.weekday(), Weekday::Tuesday);
        assert_eq!(end_of_january.into_components(), (2023, 1, 31));
    }
}
