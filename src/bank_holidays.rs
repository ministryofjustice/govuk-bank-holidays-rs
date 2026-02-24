use std::cmp::Ordering;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::PlainDate;
use crate::dates::DateWrapper;

/// Details of a bank holiday.
#[derive(Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct BankHoliday<Date: PlainDate> {
    #[serde(deserialize_with = "DateWrapper::deserialize", serialize_with = "DateWrapper::serialize")]
    date: DateWrapper<Date>,
    title: String,
    notes: String,
    bunting: bool,
}

impl<Date: PlainDate> BankHoliday<Date> {
    /// New bank holiday with blank notes and no bunting.
    #[inline]
    pub fn new(date: Date, title: String) -> Self {
        Self::new_with_notes(date, title, "".to_owned())
    }

    /// New bank holiday with notes and no bunting.
    #[inline]
    pub fn new_with_notes(date: Date, title: String, notes: String) -> Self {
        Self {
            date: DateWrapper(date),
            title,
            notes,
            bunting: false,
        }
    }

    /// Convert bank holiday into underlying date.
    #[inline]
    pub fn into_date(self) -> Date {
        self.date.0
    }

    /// Date of this bank holiday.
    #[inline]
    pub fn date(&self) -> &Date {
        &self.date
    }

    /// Title of this bank holiday.
    #[inline]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Notes such as “Substitute day”; typically blank.
    #[inline]
    pub fn notes(&self) -> &str {
        &self.notes
    }

    /// Bunting (???)
    #[inline]
    pub fn bunting(&self) -> bool {
        self.bunting
    }
}

impl<Date: PlainDate> AsRef<Date> for BankHoliday<Date> {
    #[inline]
    fn as_ref(&self) -> &Date {
        &self.date
    }
}

impl<Date: PlainDate> PartialOrd for BankHoliday<Date> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<Date: PlainDate> Ord for BankHoliday<Date> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.date.cmp(&other.date)
            .then_with(|| self.title.cmp(&other.title))
    }
}

impl<Date: PlainDate> fmt::Debug for BankHoliday<Date> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} - {}", self.date, self.title)?;
        if !self.notes.is_empty() {
            write!(f, " ({})", self.notes)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(any(feature = "chrono", feature = "time"))]
    fn check_bank_holiday<Date: PlainDate>() {
        use serde_json::json;

        let boxing_day = Date::try_from_components(2022, 12, 26)
            .expect("date should be valid");
        let boxing_day = BankHoliday::new(boxing_day, "Boxing Day".to_owned());
        let christmas = Date::try_from_components(2022, 12, 27)
            .expect("date should be valid");
        let christmas = BankHoliday::new_with_notes(christmas, "Christmas Day".to_owned(), "Substitute day".to_owned());

        assert!(boxing_day < christmas, "Boxing Day should be before Christmas (substitute day)");
        assert!(christmas > boxing_day, "Christmas (substitute day) should be after Boxing Day");

        assert_eq!(boxing_day.title(), "Boxing Day");
        assert_eq!(boxing_day.date().as_components(), (2022, 12, 26));
        assert_eq!(boxing_day.as_ref().as_components(), (2022, 12, 26));
        assert_eq!(boxing_day.into_date().as_components(), (2022, 12, 26));
        assert_eq!(christmas.notes(), "Substitute day");
        assert!(!christmas.bunting());
        assert_eq!(&format!("{:?}", christmas), "2022-12-27 - Christmas Day (Substitute day)");

        // language=json
        const BANK_HOLIDAY: &str = r#"{
            "date": "2022-12-27",
            "title": "Christmas Day",
            "notes": "Substitute day",
            "bunting": false
        }"#;
        let bank_holiday = serde_json::from_str::<BankHoliday<Date>>(BANK_HOLIDAY)
            .expect("failed to deserialise bank holiday");
        assert_eq!(bank_holiday, christmas);

        let christmas_json = serde_json::to_value(&christmas)
            .expect("failed to serialise bank holiday");
        let christmas_json = christmas_json.as_object()
            .expect("serialised bank holiday should be an object");
        assert_eq!(christmas_json.get("date"), Some(&json!("2022-12-27")));
        assert_eq!(christmas_json.get("title"), Some(&json!("Christmas Day")));
        assert_eq!(christmas_json.get("notes"), Some(&json!("Substitute day")));
        assert_eq!(christmas_json.get("bunting"), Some(&json!(false)));
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono() {
        check_bank_holiday::<crate::dates::chrono::DateImpl>()
    }

    #[cfg(feature = "time")]
    #[test]
    fn time() {
        check_bank_holiday::<crate::dates::time::DateImpl>()
    }
}
