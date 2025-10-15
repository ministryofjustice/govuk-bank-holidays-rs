use crate::{PlainDate, Weekday};

/// Used by [`BankHolidayCalendar`](crate::BankHolidayCalendar) to determine if a given date is work day
/// or not (typically, but not necessarily, the weekend).
pub trait WorkDays<Date: PlainDate> {
    /// Whether given `date` is a work day or not.
    fn is_work_day(&self, date: &Date) -> bool;
}

/// Typical working week, Monday to Friday.
#[derive(Debug, Copy, Clone)]
pub struct MonToFriWorkDays;

impl<Date: PlainDate> WorkDays<Date> for MonToFriWorkDays {
    fn is_work_day(&self, date: &Date) -> bool {
        let weekday = date.weekday();
        weekday != Weekday::Saturday && weekday != Weekday::Sunday
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(feature = "chrono", feature = "time"))]
    fn work_days_january_2024<Date: PlainDate, W: WorkDays<Date>>(w: W) -> impl Iterator<Item = bool> {
        let mut date = Date::try_from_components(2024, 1, 1)
            .expect("date should be valid");
        std::iter::from_fn(move || {
            let is_weekday = w.is_work_day(&date);
            date = date.next_day();
            Some(is_weekday)
        })
    }

    #[cfg(any(feature = "chrono", feature = "time"))]
    fn mon_to_fri<Date: PlainDate>() {
        let work_days = work_days_january_2024::<Date, MonToFriWorkDays>(MonToFriWorkDays).take(31);
        let expected = [true, true, true, true, true, false, false].iter().copied().cycle().take(31);
        assert!(work_days.eq(expected));
    }

    #[cfg(any(feature = "chrono", feature = "time"))]
    pub struct PartTime;

    #[cfg(any(feature = "chrono", feature = "time"))]
    impl<Date: PlainDate> WorkDays<Date> for PartTime {
        fn is_work_day(&self, date: &Date) -> bool {
            matches!(date.weekday(), Weekday::Monday | Weekday::Tuesday | Weekday::Wednesday)
        }
    }

    #[cfg(any(feature = "chrono", feature = "time"))]
    fn custom_work_days<Date: PlainDate>() {
        let work_days = work_days_january_2024::<Date, PartTime>(PartTime).take(8);
        let expected = [true, true, true, false, false, false, false, true];
        assert!(work_days.eq(expected));
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono() {
        mon_to_fri::<crate::dates::chrono::DateImpl>();
        custom_work_days::<crate::dates::chrono::DateImpl>();
    }

    #[cfg(feature = "time")]
    #[test]
    fn time() {
        mon_to_fri::<crate::dates::time::DateImpl>();
        custom_work_days::<crate::dates::time::DateImpl>();
    }
}
