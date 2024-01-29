use std::collections::{HashMap, HashSet};
use std::iter::FusedIterator;

use crate::{BankHoliday, Date, Division, Error, MonToFriWorkDays, WorkDays};
use crate::data_source::{Cached, DataSource, LoadDataSource, Reqwest};

/// Calendar of known bank holidays.
///
/// NB: Bank holidays vary between parts of the UK so GOV.UK provide separate lists for different “divisions”.
/// Methods taking an [`Option<Division>`](Division) parameter will only consider bank holidays common to
/// *all* divisions if `None` is provided.
pub struct BankHolidayCalendar<W: WorkDays> {
    holiday_map: HashMap<Division, Vec<BankHoliday>>,
    holidays_common_to_all_divisions: HashSet<Date>,
    work_days: W,
}

impl BankHolidayCalendar<MonToFriWorkDays> {
    /// Load UK bank holidays from [GOV.UK](https://www.gov.uk/bank-holidays), falling back to cached/embedded data,
    /// using a Monday to Friday work week.
    #[inline]
    pub async fn load() -> Self {
        Self::load_with(MonToFriWorkDays).await
    }

    /// Build from cached/embedded data, using a Monday to Friday work week.
    #[inline]
    pub fn cached() -> Self {
        Self::cached_with(MonToFriWorkDays)
    }

    /// Build with a custom source of bank holidays, using a Monday to Friday work week.
    #[inline]
    pub async fn custom<T: LoadDataSource>(loader: T) -> Result<Self, Error> {
        Self::custom_with(loader, MonToFriWorkDays).await
    }
}

impl<W: WorkDays> BankHolidayCalendar<W> {
    /// Load UK bank holidays from [GOV.UK](https://www.gov.uk/bank-holidays), falling back to cached/embedded data,
    /// using given [`WorkDays`].
    pub async fn load_with(work_days: W) -> Self {
        let data_source = Reqwest::default().load_data_source().await.unwrap_or_else(|error| {
            tracing::error!("Failed to load bank holidays: {error}");
            tracing::info!("Falling back to cached calendar data");
            Cached::cached_data_source()
        });
        Self::new(data_source, work_days)
    }

    /// Build from cached/embedded data, using given [`WorkDays`].
    #[inline]
    pub fn cached_with(work_days: W) -> Self {
        Self::new(Cached::cached_data_source(), work_days)
    }

    /// Build with a custom source of bank holidays, using given [`WorkDays`].
    pub async fn custom_with<T: LoadDataSource>(loader: T, work_days: W) -> Result<Self, Error> {
        loader.load_data_source().await
            .map(|data_source| Self::new(data_source, work_days))
    }

    /// Private method to build a calendar from a [`DataSource`] and given [`WorkDays`].
    fn new(data_source: DataSource, work_days: W) -> Self {
        let holiday_map = data_source.into_inner();
        let holidays_common_to_all_divisions = holiday_map.values()
            .fold(None, |common: Option<HashSet<Date>>, bank_holidays| {
                let dates: HashSet<_> = bank_holidays.iter()
                    .map(|bank_holiday| bank_holiday.date)
                    .collect();
                if let Some(common) = common {
                    let common: HashSet<_> = common.intersection(&dates)
                        .copied()
                        .collect();
                    Some(common)
                } else {
                    Some(dates)
                }
            })
            .unwrap_or_else(|| {
                tracing::warn!("Empty bank holiday calendar");
                HashSet::new()
            });
        BankHolidayCalendar { holiday_map, holidays_common_to_all_divisions, work_days }
    }

    /// Get all known holidays in given `division` of the UK or only those common to all divisions.
    pub fn holidays(&self, division: Option<Division>) -> Vec<&BankHoliday> {
        let mut holidays = Vec::new();
        if let Some(division) = division {
            if let Some(bank_holidays) = self.holiday_map.get(&division) {
                holidays.extend(bank_holidays);
            }
        } else {
            let bank_holidays = self.holiday_map.get(&Division::EnglandAndWales)
                .or_else(|| self.holiday_map.values().next());
            if let Some(bank_holidays) = bank_holidays {
                holidays.extend(
                    bank_holidays.iter()
                        .filter(|bank_holiday| {
                            self.holidays_common_to_all_divisions.contains(&bank_holiday.date)
                        })
                );
            }
        }
        holidays
    }

    /// Checks whether `date` is a bank holiday in given `division` or common to all divisions.
    pub fn is_holiday(&self, date: Date, division: Option<Division>) -> bool {
        if let Some(division) = division {
            self.holiday_map.get(&division)
                .map(|bank_holidays| {
                    bank_holidays.iter()
                        .any(|bank_holiday| bank_holiday.date == date)
                })
                .unwrap_or(false)
        } else {
            self.holidays_common_to_all_divisions.contains(&date)
        }
    }

    /// Checks whether `date` is a work day in given `division` or common to all divisions.
    pub fn is_work_day(&self, date: Date, division: Option<Division>) -> bool {
        self.work_days.is_work_day(date) && !self.is_holiday(date, division)
    }

    /// Get [`WorkDays`] implementation.
    #[inline]
    pub fn work_days(&self) -> &W {
        &self.work_days
    }

    /// Get mutable [`WorkDays`] implementation.
    #[inline]
    pub fn work_days_mut(&mut self) -> &mut W {
        &mut self.work_days
    }

    /// Iterate over all known bank holidays _after_ a `date` in given `division` or common to all divisions.
    /// Iterator yields [`&BankHoliday`](BankHoliday).
    pub fn iter_holidays_after(&self, date: Date, division: Option<Division>) -> HolidayIter<'_> {
        let holidays = self.holidays(division)
            .drain(..)
            .rev()
            .filter(|bank_holiday| bank_holiday.date > date)
            .collect();
        HolidayIter { holidays }
    }

    /// Iterate over all known bank holidays _before_ a `date` in given `division` or common to all divisions.
    /// Iterator yields [`&BankHoliday`](BankHoliday).
    pub fn iter_holidays_before(&self, date: Date, division: Option<Division>) -> HolidayIter<'_> {
        let holidays = self.holidays(division)
            .drain(..)
            .filter(|bank_holiday| bank_holiday.date < date)
            .collect();
        HolidayIter { holidays }
    }

    /// Iterate over all work days _after_ a `date`, skipping bank holidays in given `division`
    /// or common to all divisions.
    /// Iterator yields [`Date`].
    ///
    /// NB: this is an infinite iterator.
    #[inline]
    pub fn iter_work_days_after(&self, date: Date, division: Option<Division>) -> WorkDayIter<'_, W> {
        WorkDayIter { calendar: self, date, division, forward: true }
    }

    /// Iterate over all work days _before_ a `date`, skipping bank holidays in given `division`
    /// or common to all divisions.
    /// Iterator yields [`Date`].
    ///
    /// NB: this is an infinite iterator.
    #[inline]
    pub fn iter_work_days_before(&self, date: Date, division: Option<Division>) -> WorkDayIter<'_, W> {
        WorkDayIter { calendar: self, date, division, forward: false }
    }
}

pub struct HolidayIter<'a> {
    holidays: Vec<&'a BankHoliday>,
}

impl<'a> Iterator for HolidayIter<'a> {
    type Item = &'a BankHoliday;

    fn next(&mut self) -> Option<Self::Item> {
        self.holidays.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.holidays.len(), Some(self.holidays.len()))
    }
}

impl<'a> ExactSizeIterator for HolidayIter<'a> {}

impl<'a> FusedIterator for HolidayIter<'a> {}

pub struct WorkDayIter<'a, W: WorkDays> {
    calendar: &'a BankHolidayCalendar<W>,
    date: Date,
    division: Option<Division>,
    forward: bool,
}

impl<'a, W: WorkDays> WorkDayIter<'a, W> {
    fn advance_date(&mut self) {
        if self.forward {
            self.date = self.date.next_day()
        } else {
            self.date = self.date.previous_day()
        }
    }
}

impl<'a, W: WorkDays> Iterator for WorkDayIter<'a, W> {
    type Item = Date;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_date();
        while !self.calendar.is_work_day(self.date, self.division) {
            self.advance_date();
        }
        Some(self.date)
    }
}
