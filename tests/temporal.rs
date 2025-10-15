//! Example of how `PlainDate` can be implemented on `temporal_rs::PlainDate`

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use govuk_bank_holidays::{Error, Weekday};
use govuk_bank_holidays::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(transparent)]
struct Wrapper(temporal_rs::PlainDate);

impl From<temporal_rs::PlainDate> for Wrapper {
    #[inline]
    fn from(date: temporal_rs::PlainDate) -> Self {
        Wrapper(date)
    }
}

impl Deref for Wrapper {
    type Target = temporal_rs::PlainDate;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialOrd for Wrapper {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Wrapper {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_iso(other)
    }
}

impl Hash for Wrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.0.year(), state);
        Hash::hash(&self.0.month(), state);
        Hash::hash(&self.0.day(), state);
    }
}

impl PlainDate for Wrapper {
    fn try_from_components(year: i32, month: u8, day: u8) -> Result<Self, Error> {
        temporal_rs::PlainDate::try_new_iso(year, month, day)
            .map(Wrapper)
            .map_err(|_| Error::InvalidDate)
    }

    #[inline]
    fn as_components(&self) -> (i32, u8, u8) {
        (self.0.year(), self.0.month(), self.0.day())
    }

    #[inline]
    fn year(&self) -> i32 {
        self.0.year()
    }

    #[inline]
    fn month(&self) -> u8 {
        self.0.month()
    }

    #[inline]
    fn day(&self) -> u8 {
        self.0.day()
    }

    fn weekday(&self) -> Weekday {
        match self.0.day_of_week() {
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            7 => Weekday::Sunday,
            _ => unreachable!()
        }
    }

    fn previous_day(&self) -> Self {
        let one_day = temporal_rs::partial::PartialDuration::empty().with_days(-1).try_into().unwrap();
        let date = self.0.add(&one_day, None).expect("date should exist");
        Wrapper(date)
    }

    fn next_day(&self) -> Self {
        let one_day = temporal_rs::partial::PartialDuration::empty().with_days(1).try_into().unwrap();
        let date = self.0.add(&one_day, None).expect("date should exist");
        Wrapper(date)
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
