//! Example of how `PlainDate` can be implemented on `jiff::civil::Date`

use std::ops::Deref;

use govuk_bank_holidays::{Error, Weekday};
use govuk_bank_holidays::prelude::*;
use jiff::ToSpan;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
struct Wrapper(jiff::civil::Date);

impl From<jiff::civil::Date> for Wrapper {
    #[inline]
    fn from(date: jiff::civil::Date) -> Self {
        Wrapper(date)
    }
}

impl Deref for Wrapper {
    type Target = jiff::civil::Date;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PlainDate for Wrapper {
    fn try_from_components(year: i32, month: u8, day: u8) -> Result<Self, Error> {
        jiff::civil::Date::new(
            year.try_into().map_err(|_| Error::InvalidDate)?,
            month.try_into().map_err(|_| Error::InvalidDate)?,
            day.try_into().map_err(|_| Error::InvalidDate)?,
        )
            .map(Wrapper)
            .map_err(|_| Error::InvalidDate)
    }

    #[inline]
    fn as_components(&self) -> (i32, u8, u8) {
        (self.0.year() as i32, self.0.month() as u8, self.0.day() as u8)
    }

    #[inline]
    fn year(&self) -> i32 {
        self.0.year() as i32
    }

    #[inline]
    fn month(&self) -> u8 {
        self.0.month() as u8
    }

    #[inline]
    fn day(&self) -> u8 {
        self.0.day() as u8
    }

    fn weekday(&self) -> Weekday {
        match self.0.weekday() {
            jiff::civil::Weekday::Monday => Weekday::Monday,
            jiff::civil::Weekday::Tuesday => Weekday::Tuesday,
            jiff::civil::Weekday::Wednesday => Weekday::Wednesday,
            jiff::civil::Weekday::Thursday => Weekday::Thursday,
            jiff::civil::Weekday::Friday => Weekday::Friday,
            jiff::civil::Weekday::Saturday => Weekday::Saturday,
            jiff::civil::Weekday::Sunday => Weekday::Sunday,
        }
    }

    fn previous_day(&self) -> Self {
        Wrapper(self.0 - 1.day())
    }

    fn next_day(&self) -> Self {
        Wrapper(self.0 + 1.day())
    }
}

#[test]
fn simple() {
    let calendar: BankHolidayCalendar<Wrapper, _> = BankHolidayCalendar::cached();
    let holiday = calendar.iter_holidays_before(&Wrapper::try_from_components(2023, 10, 19).unwrap(), None)
        .next()
        .expect("there should be a bank holiday")
        .as_ref();
    assert_eq!(holiday.year(), 2023);
    assert_eq!(holiday.month(), 5);
    assert_eq!(holiday.day(), 29);
}
