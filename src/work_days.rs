use crate::{Date, Weekday};

/// Used by [`BankHolidayCalendar`](crate::BankHolidayCalendar) to determine if a given date is work day
/// or not (typically, but not necessarily, the weekend).
pub trait WorkDays {
    /// Whether given `date` is a work day or not.
    fn is_work_day(&self, date: Date) -> bool;
}

/// Typical working week, Monday to Friday.
#[derive(Debug, Copy, Clone)]
pub struct MonToFriWorkDays;

impl WorkDays for MonToFriWorkDays {
    fn is_work_day(&self, date: Date) -> bool {
        let weekday = date.weekday();
        weekday != Weekday::Saturday && weekday != Weekday::Sunday
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mon_to_fri() {
        let mut date = Date::try_from_components(2024, 1, 1)
            .expect("date should be valid");
        let weekdays_january_2024 = std::iter::from_fn(move || {
            let is_weekday = WorkDays::is_work_day(&MonToFriWorkDays, date);
            date = date.next_day();
            Some(is_weekday)
        }).take(31);
        let expected = [true, true, true, true, true, false, false].iter().copied().cycle().take(31);
        assert!(weekdays_january_2024.eq(expected)); // naturally ignores holidays
    }
}
