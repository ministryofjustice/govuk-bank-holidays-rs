//! # GOV.UK Bank Holidays
//!
//! This library loads the official list of bank holidays in the United Kingdom as supplied by
//! [GOV.UK](https://www.gov.uk/bank-holidays), which tends to provide this list for only a year or two into the future.
//!
//! A cached backup list of known bank holidays is stored in this package, though it is not updated often.
//! GOV.UK no longer provide bank holidays for some of the older years still part of this backup list.
//!
//! Bank holidays differ around the UK. The GOV.UK source currently lists these for 3 “divisions”:
//!
//! - England and Wales
//! - Scotland
//! - Northern Ireland
//!
//! Methods on [`BankHolidayCalendar`] that take a `division` parameter will consider bank holidays only for the provided
//! division ([`Some(Division)`](Division)) or only those that are **common** to all divisions for `None`.
//!
//! ## Usage
//!
//! ```no_run
//! use govuk_bank_holidays::prelude::*;
//!
//! # async fn demo() {
//! let date = Date::today();
//!
//! // load bank holidays from GOV.UK
//! let calendar = BankHolidayCalendar::load().await;
//!
//! // check if the given date is a bank holiday in _all_ divisions
//! let is_holiday = calendar.is_holiday(date, None);
//! println!("Is {date} a bank holiday across the UK? {is_holiday}");
//!
//! // check if the given date is a work day in Northern Ireland
//! let is_work_day = calendar.is_work_day(date, Some(Division::NorthernIreland));
//! println!("Is {date} a work day in NI? {is_work_day}");
//! # }
//! ```
//!
//! ## Features
//!
//! The date implementation is swappable:
//!
//! - default or `"chrono"` will use the [`chrono`](https://crates.io/crates/chrono) crate
//! - `"time"` will use the [`time`](https://crates.io/crates/time) crate
//!
//! `chrono` and `time` cannot be used together.

mod bank_holidays;
mod calendar;
pub mod data_source;
mod dates;
mod divisions;
mod errors;
mod work_days;

pub use bank_holidays::BankHoliday;
pub use calendar::BankHolidayCalendar;
pub use dates::{Date, Weekday};
pub use divisions::Division;
pub use errors::Error;
pub use work_days::{WorkDays, MonToFriWorkDays};

/// Commonly-used items.
///
/// ```no_run
/// use govuk_bank_holidays::prelude::*;
/// ```
pub mod prelude {
    pub use super::{BankHolidayCalendar, Date, Division};
}

/// Default URL to load bank holidays from.
pub const SOURCE_URL: &str = "https://www.gov.uk/bank-holidays.json";

#[cfg(test)]
mod tests {
    use super::*;

    fn test(calendar: BankHolidayCalendar<MonToFriWorkDays>) {
        let date = Date::try_from_components(2023, 1, 10)
            .expect("date should be valid");
        assert!(calendar.is_work_day(date, None));
        assert!(!calendar.is_holiday(date, None));

        let date = Date::try_from_components(2022, 1, 1)
            .expect("date should be valid");
        let holidays = calendar.iter_holidays_after(date, Some(Division::EnglandAndWales));
        for bank_holiday in holidays {
            // eprintln!("Holidays after 1/1/22: {:?}", bank_holiday);
            assert!(bank_holiday.date > date);
        }

        let date = Date::try_from_components(2022, 1, 7)
            .expect("date should be valid");
        let mut work_days = calendar.iter_work_days_before(date, Some(Division::EnglandAndWales));
        for _ in 0..10 {
            let work_day = work_days.next().unwrap();
            // eprintln!("10 work days before 7/1/22: {work_day}");
            assert!(work_day < date);
            assert!(calendar.is_work_day(work_day, Some(Division::EnglandAndWales)));
        }
    }

    #[test]
    fn cached() {
        test(BankHolidayCalendar::cached());
    }

    #[tokio::test]
    #[ignore]
    async fn requested() {
        test(BankHolidayCalendar::load().await);
    }

    #[tokio::test]
    async fn custom_work_days() {
        struct PartTime;

        impl WorkDays for PartTime {
            fn is_work_day(&self, date: Date) -> bool {
                matches!(date.weekday(), Weekday::Monday | Weekday::Tuesday | Weekday::Wednesday)
            }
        }

        let calendar = BankHolidayCalendar::cached_with(PartTime);

        // check days of whole month are correctly flagged as work days or not
        let mut date = Date::try_from_components(2024, 2, 1)
            .expect("date should be valid");
        let work_days_february_2024 = std::iter::from_fn(move || {
            let is_work_day = calendar.is_work_day(date, Some(Division::Scotland));
            date = date.next_day();
            Some(is_work_day)
        }).take(29);
        let expected = [false, false, false, false, true, true, true].iter().copied().cycle().take(29);
        assert!(work_days_february_2024.eq(expected)); // there are no bank holidays in Scotland this month
    }

    #[test]
    fn next_holiday() {
        let calendar = BankHolidayCalendar::cached();
        let date = Date::try_from_components(2016, 1, 2).unwrap();

        let mut holidays = calendar.iter_holidays_after(date, None).
            map(|holiday| holiday.date.into_components());
        assert_eq!(holidays.next(), Some((2016, 3, 25)));
        assert_eq!(holidays.next(), Some((2016, 5, 2)));
        assert_eq!(holidays.next(), Some((2016, 5, 30)));

        let mut holidays = calendar.iter_holidays_after(date, Some(Division::Scotland))
            .map(|holiday| holiday.date.into_components());
        assert_eq!(holidays.next(), Some((2016, 1, 4)));
        assert_eq!(holidays.next(), Some((2016, 3, 25)));
        assert_eq!(holidays.next(), Some((2016, 5, 2)));
    }

    #[test]
    fn previous_holiday() {
        let calendar = BankHolidayCalendar::cached();
        let date = Date::try_from_components(2016, 1, 5).unwrap();

        let mut holidays = calendar.iter_holidays_before(date, None)
            .map(|holiday| holiday.date.into_components());
        assert_eq!(holidays.next(), Some((2016, 1, 1)));
        assert_eq!(holidays.next(), Some((2015, 12, 28)));
        assert_eq!(holidays.next(), Some((2015, 12, 25)));

        let mut holidays = calendar.iter_holidays_before(date, Some(Division::Scotland))
            .map(|holiday| holiday.date.into_components());
        assert_eq!(holidays.next(), Some((2016, 1, 4)));
        assert_eq!(holidays.next(), Some((2016, 1, 1)));
        assert_eq!(holidays.next(), Some((2015, 12, 28)));
    }

    #[test]
    fn holiday_check() {
        let calendar = BankHolidayCalendar::cached();
        let is_holiday = calendar.is_holiday(Date::try_from_components(2012, 1, 2).unwrap(), None);
        assert!(is_holiday);
        let is_holiday = calendar.is_holiday(Date::try_from_components(2016, 1, 4).unwrap(), None);
        assert!(!is_holiday);
        let is_holiday = calendar.is_holiday(Date::try_from_components(2016, 1, 4).unwrap(), Some(Division::Scotland));
        assert!(is_holiday);
    }

    #[test]
    fn next_work_day() {
        let calendar = BankHolidayCalendar::cached();

        let mut work_days = calendar.iter_work_days_after(Date::try_from_components(2017, 12, 19).unwrap(), None)
            .map(Date::into_components);
        assert_eq!(work_days.next(), Some((2017, 12, 20)));
        assert_eq!(work_days.next(), Some((2017, 12, 21)));
        assert_eq!(work_days.next(), Some((2017, 12, 22)));
        assert_eq!(work_days.next(), Some((2017, 12, 27)));
        assert_eq!(work_days.next(), Some((2017, 12, 28)));
        assert_eq!(work_days.next(), Some((2017, 12, 29)));
        assert_eq!(work_days.next(), Some((2018, 1, 2)));
        assert_eq!(work_days.next(), Some((2018, 1, 3)));

        let mut work_days = calendar.iter_work_days_after(Date::try_from_components(2017, 12, 19).unwrap(), Some(Division::Scotland))
            .map(Date::into_components);
        assert_eq!(work_days.next(), Some((2017, 12, 20)));
        assert_eq!(work_days.next(), Some((2017, 12, 21)));
        assert_eq!(work_days.next(), Some((2017, 12, 22)));
        assert_eq!(work_days.next(), Some((2017, 12, 27)));
        assert_eq!(work_days.next(), Some((2017, 12, 28)));
        assert_eq!(work_days.next(), Some((2017, 12, 29)));
        assert_eq!(work_days.next(), Some((2018, 1, 3)));
    }

    #[test]
    fn previous_work_day() {
        let calendar = BankHolidayCalendar::cached();

        let mut work_days = calendar.iter_work_days_before(Date::try_from_components(2018, 1, 3).unwrap(), None)
            .map(Date::into_components);
        assert_eq!(work_days.next(), Some((2018, 1, 2)));
        assert_eq!(work_days.next(), Some((2017, 12, 29)));
        assert_eq!(work_days.next(), Some((2017, 12, 28)));
        assert_eq!(work_days.next(), Some((2017, 12, 27)));
        assert_eq!(work_days.next(), Some((2017, 12, 22)));
        assert_eq!(work_days.next(), Some((2017, 12, 21)));
        assert_eq!(work_days.next(), Some((2017, 12, 20)));
        assert_eq!(work_days.next(), Some((2017, 12, 19)));

        let mut work_days = calendar.iter_work_days_before(Date::try_from_components(2018, 1, 3).unwrap(), Some(Division::Scotland))
            .map(Date::into_components);
        assert_eq!(work_days.next(), Some((2017, 12, 29)));
        assert_eq!(work_days.next(), Some((2017, 12, 28)));
        assert_eq!(work_days.next(), Some((2017, 12, 27)));
        assert_eq!(work_days.next(), Some((2017, 12, 22)));
        assert_eq!(work_days.next(), Some((2017, 12, 21)));
        assert_eq!(work_days.next(), Some((2017, 12, 20)));
        assert_eq!(work_days.next(), Some((2017, 12, 19)));
    }

    fn holidays_2018_to_2022(bank_holiday: &BankHoliday) -> bool {
        bank_holiday.date.year() >= 2018 && bank_holiday.date.year() <= 2022
    }

    #[test]
    fn number_of_bank_holidays() {
        let calendar = BankHolidayCalendar::cached();
        let expectation = [
            (None, 32),
            (Some(Division::EnglandAndWales), 42),
            (Some(Division::Scotland), 47),
            (Some(Division::NorthernIreland), 52),
        ];
        for (division, expected_count) in expectation {
            let holiday_count = calendar.holidays(division)
                .into_iter()
                .filter(|holiday| holidays_2018_to_2022(holiday))
                .count();
            assert_eq!(holiday_count, expected_count, "Unexpected number of bank holidays in {:?}", division);

            let end_of_2017 = Date::try_from_components(2017, 12, 31).unwrap();
            let holiday_count = calendar.iter_holidays_after(end_of_2017, division)
                .filter(|holiday| holidays_2018_to_2022(holiday))
                .count();
            assert_eq!(holiday_count, expected_count, "Unexpected number of bank holidays in {:?}", division);

            let start_of_2023 = Date::try_from_components(2023, 1, 1).unwrap();
            let holiday_count = calendar.iter_holidays_before(start_of_2023, division)
                .filter(|holiday| holidays_2018_to_2022(holiday))
                .count();
            assert_eq!(holiday_count, expected_count, "Unexpected number of bank holidays in {:?}", division);
        }
    }

    #[test]
    fn bank_holidays_in_divisions() {
        let calendar = BankHolidayCalendar::cached();
        let expectation = [
            (None, "Christmas Day", true),
            (Some(Division::EnglandAndWales), "Christmas Day", true),
            (Some(Division::Scotland), "Christmas Day", true),
            (Some(Division::NorthernIreland), "Christmas Day", true),
            (None, "St Patrick’s Day", false),
            (Some(Division::EnglandAndWales), "St Patrick’s Day", false),
            (Some(Division::NorthernIreland), "St Patrick’s Day", true),
            (None, "St Andrew’s Day", false),
            (Some(Division::EnglandAndWales), "St Andrew’s Day", false),
            (Some(Division::Scotland), "St Andrew’s Day", true),
        ];

        for (division, title, expect_exists) in expectation {
            let bank_holiday_found = calendar.holidays(division).iter()
                .filter(|holiday| holidays_2018_to_2022(holiday))
                .any(|holiday| holiday.title.as_str() == title);
            assert_eq!(bank_holiday_found, expect_exists, "Expect “{title}” to exist in {:?}: {expect_exists}", division);
        }
    }
}
