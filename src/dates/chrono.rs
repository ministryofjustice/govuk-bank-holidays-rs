use chrono::Datelike;

use crate::{Error, PlainDate, Weekday};

/// [PlainDate] implementation using the `chrono::NaiveDate` naive date type.
pub(crate) type DateImpl = chrono::NaiveDate;

impl PlainDate for DateImpl {
    fn try_from_components(year: i32, month: u8, day: u8) -> Result<Self, Error> {
        DateImpl::from_ymd_opt(year, month as u32, day as u32)
            .ok_or(Error::InvalidDate)
    }

    #[inline]
    fn as_components(&self) -> (i32, u8, u8) {
        (
            Datelike::year(self),
            Datelike::month(self) as u8,
            Datelike::day(self) as u8,
        )
    }

    #[inline]
    fn year(&self) -> i32 {
        Datelike::year(self)
    }

    #[inline]
    fn month(&self) -> u8 {
        Datelike::month(self) as u8
    }

    #[inline]
    fn day(&self) -> u8 {
        Datelike::day(self) as u8
    }

    fn weekday(&self) -> Weekday {
        match Datelike::weekday(self) {
            chrono::Weekday::Mon => Weekday::Monday,
            chrono::Weekday::Tue => Weekday::Tuesday,
            chrono::Weekday::Wed => Weekday::Wednesday,
            chrono::Weekday::Thu => Weekday::Thursday,
            chrono::Weekday::Fri => Weekday::Friday,
            chrono::Weekday::Sat => Weekday::Saturday,
            chrono::Weekday::Sun => Weekday::Sunday,
        }
    }

    fn previous_day(&self) -> Self {
        *self - chrono::Duration::days(1)
    }

    fn next_day(&self) -> Self {
        *self + chrono::Duration::days(1)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;
    use crate::dates::DateWrapper;
    use crate::dates::tests::{check_plain_date_impl, check_wrapped_date};

    #[test]
    fn test_impl() {
        // natively constructed chrono:NaiveDate
        let date = DateImpl::from_ymd_opt(2024, 2, 29)
            .expect("date should be valid");
        check_plain_date_impl(&date);

        // chrono:NaiveDate constructed via PlainDate trait
        let date = DateImpl::try_from_components(2024, 2, 29)
            .expect("date should be valid");
        check_plain_date_impl(&date);

        // via private newtype wrapper
        let date: DateWrapper<DateImpl> = date.into();
        check_plain_date_impl(date.deref());
        check_plain_date_impl(&date.0);
    }

    #[test]
    fn test_serde() {
        let today = chrono::Local::now().date_naive();
        let today = DateWrapper(today);
        check_wrapped_date(today);
    }
}
