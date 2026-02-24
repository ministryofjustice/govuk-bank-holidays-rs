use crate::{Error, PlainDate, Weekday};

/// [PlainDate] implementation using the `time::Date` naive date type.
pub(crate) type DateImpl = time::Date;

impl PlainDate for DateImpl {
    fn try_from_components(year: i32, month: u8, day: u8) -> Result<Self, Error> {
        DateImpl::from_calendar_date(year, month.try_into().map_err(|_| Error::InvalidDate)?, day)
            .map_err(|_| Error::InvalidDate)
    }

    #[inline]
    fn as_components(&self) -> (i32, u8, u8) {
        (
            DateImpl::year(*self),
            DateImpl::month(*self) as u8,
            DateImpl::day(*self),
        )
    }

    #[inline]
    fn year(&self) -> i32 {
        DateImpl::year(*self)
    }

    #[inline]
    fn month(&self) -> u8 {
        DateImpl::month(*self) as u8
    }

    #[inline]
    fn day(&self) -> u8 {
        DateImpl::day(*self)
    }

    fn weekday(&self) -> Weekday {
        match DateImpl::weekday(*self) {
            time::Weekday::Monday => Weekday::Monday,
            time::Weekday::Tuesday => Weekday::Tuesday,
            time::Weekday::Wednesday => Weekday::Wednesday,
            time::Weekday::Thursday => Weekday::Thursday,
            time::Weekday::Friday => Weekday::Friday,
            time::Weekday::Saturday => Weekday::Saturday,
            time::Weekday::Sunday => Weekday::Sunday,
        }
    }

    fn previous_day(&self) -> Self {
        *self - time::Duration::days(1)
    }

    fn next_day(&self) -> Self {
        *self + time::Duration::days(1)
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
        // natively constructed time::Date
        let date = DateImpl::from_calendar_date(2024, time::Month::February, 29)
            .expect("date should be valid");
        check_plain_date_impl(&date);

        // time::Date constructed via PlainDate trait
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
        let today = time::OffsetDateTime::now_local()
            .expect("cannot get now in local timezone").date();
        let today = DateWrapper(today);
        check_wrapped_date(today);
    }
}
