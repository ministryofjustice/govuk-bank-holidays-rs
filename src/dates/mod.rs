use std::fmt;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Deref;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::Error;

#[cfg(feature = "chrono")]
/// [PlainDate] implementation using the `chrono` library.
pub(crate) mod chrono;
#[cfg(feature = "time")]
/// [PlainDate] implementation using the `time` library.
pub(crate) mod time;

/// Represents a date in the Gregorian calendar without time zone information.
pub trait PlainDate: Clone + Hash + Eq + Ord + fmt::Debug {
    /// Create a date from day, month and year integers.
    // NB: it’s probably this method that makes the trait not dyn compatible.
    fn try_from_components(year: i32, month: u8, day: u8) -> Result<Self, Error>;

    /// Convert into year, month and day.
    fn as_components(&self) -> (i32, u8, u8);

    /// Get the date’s year.
    fn year(&self) -> i32;

    /// Get the date’s month.
    fn month(&self) -> u8;

    /// Get the date’s day.
    fn day(&self) -> u8;

    /// Get the date’s day of the week.
    fn weekday(&self) -> Weekday;

    /// Format into an ISO 8601 date string, like "YYYY-MM-DD".
    fn iso_date_string(&self) -> String {
        format!("{}-{:02}-{:02}", self.year(), self.month(), self.day())
    }

    /// Preceding date.
    fn previous_day(&self) -> Self;

    /// Following date.
    fn next_day(&self) -> Self;
}

/// A day of the week from the Gregorian calendar.
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
#[repr(u8)]
pub enum Weekday {
    Monday = 1,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Private newtype used internally for consistent de/serialisation.
/// Easily constructed from inner date implementation and dereferences to it so has all `PlainDate` methods.
#[derive(PartialEq, Eq, Hash, Clone)]
pub(crate) struct DateWrapper<D: PlainDate>(pub(crate) D);

impl<Date: PlainDate> From<Date> for DateWrapper<Date> {
    #[inline]
    fn from(date: Date) -> Self {
        DateWrapper(date)
    }
}

impl<Date: PlainDate> Deref for DateWrapper<Date> {
    type Target = Date;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Date: PlainDate> fmt::Debug for DateWrapper<Date> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl<Date: PlainDate + fmt::Display> fmt::Display for DateWrapper<Date> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<'de, Date: PlainDate> Deserialize<'de> for DateWrapper<Date> {
    fn deserialize<De: Deserializer<'de>>(deserializer: De) -> Result<Self, De::Error> {
        struct Visitor<Date>(PhantomData<fn() -> Date>);

        impl<'de, Date: PlainDate> de::Visitor<'de> for Visitor<Date> {
            type Value = DateWrapper<Date>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string with ISO 8601 formatted naive date")
            }

            fn visit_str<E: de::Error>(self, input: &str) -> Result<Self::Value, E> {
                let input = input.as_bytes();

                fn consume_char<const C: char, E: de::Error>(input: &[u8]) -> Result<&[u8], E> {
                    match input {
                        [] => Err(E::custom("expected separator")),
                        [c, remaining @ ..] if *c == C as u8 => Ok(remaining),
                        _ => Err(E::custom("unexpected character")),
                    }
                }

                fn consume_integer<E: de::Error>(mut input: &[u8]) -> Result<(i32, &[u8]), E> {
                    let mut integer: Option<i32> = None;
                    loop {
                        match input {
                            [c, remaining @ ..] if c.is_ascii_digit() => {
                                input = remaining;
                                let digit = (*c - b'0') as i32;
                                integer = match integer {
                                    Some(integer) => Some(
                                        integer.checked_mul(10).and_then(|integer| integer.checked_add(digit))
                                            .ok_or_else(|| E::custom("unexpected integer"))?
                                    ),
                                    None => Some(digit),
                                };
                            }
                            _ => return integer.map(|integer| (integer, input))
                                .ok_or_else(|| E::custom("integer not found")),
                        }
                    }
                }

                let (year, input) = consume_integer::<E>(input)?;
                let input = consume_char::<'-', E>(input)?;
                let (month, input) = consume_integer::<E>(input)?;
                let month = u8::try_from(month)
                    .map_err(|_| E::custom("invalid month"))?;
                let input = consume_char::<'-', E>(input)?;
                let (day, input) = consume_integer::<E>(input)?;
                let day = u8::try_from(day)
                    .map_err(|_| E::custom("invalid day"))?;
                if !input.is_empty() {
                    return Err(E::custom("unexpected suffix"));
                }

                Date::try_from_components(year, month, day)
                    .map(DateWrapper)
                    .map_err(|_| E::custom("invalid date"))
            }
        }

        deserializer.deserialize_str(Visitor(PhantomData))
    }
}

impl<Date: PlainDate> Serialize for DateWrapper<Date> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.iso_date_string())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Test `PlainDate` trait methods with implementation; call with 2024-02-29
    #[cfg(any(feature = "chrono", feature = "time"))]
    pub fn check_plain_date_impl<Date: PlainDate>(date: &Date) {
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 2);
        assert_eq!(date.day(), 29);
        assert_eq!(date.weekday(), Weekday::Thursday);
        assert_eq!(&date.iso_date_string(), "2024-02-29");
        let (year, month, day) = date.as_components();
        assert_eq!(year, 2024);
        assert_eq!(month, 2);
        assert_eq!(day, 29);

        let previous_day = date.previous_day();
        let next_day = date.next_day();
        assert_eq!(
            (previous_day.iso_date_string().as_str(), next_day.iso_date_string().as_str()),
            ("2024-02-28", "2024-03-01"),
        );
    }

    /// Test internal wrapped date methods with implementation
    #[cfg(any(feature = "chrono", feature = "time"))]
    pub fn check_wrapped_date<Date: PlainDate>(wrapped_date: DateWrapper<Date>) {
        let date_json_string = serde_json::to_string(&wrapped_date)
            .expect("failed to serialise date");
        let deserialised_wrapped_date = serde_json::from_str::<DateWrapper<Date>>(&date_json_string)
            .expect("failed to deserialise date");
        let (year, month, day) = deserialised_wrapped_date.as_components();
        let date = wrapped_date.0;
        assert_eq!(year, date.year());
        assert_eq!(month, date.month());
        assert_eq!(day, date.day());

        for (malformed_date_str, expected_error) in [
            (r#""#, "EOF"),
            (r#"null"#, "expected a string with ISO 8601 formatted naive date"),
            (r#"2024"#, "expected a string with ISO 8601 formatted naive date"),
            (r#""""#, "integer not found"),
            (r#""2024""#, "expected separator"),
            (r#""20240101""#, "expected separator"),
            (r#""2024-01""#, "expected separator"),
            (r#""2024-01-""#, "integer not found"),
            (r#""2024-01-01Z""#, "unexpected suffix"),
            (r#""2024-01-01T12:00:00""#, "unexpected suffix"),
            (r#""2024-30-01""#, "invalid date"),
            (r#""2024-01-32""#, "invalid date"),
            (r#""2024-300-01""#, "invalid month"),
            (r#""01-01-2024""#, "invalid day"),
            (r#""2024/01/01""#, "unexpected character"),
        ] {
            let actual_error = serde_json::from_str::<DateWrapper<Date>>(malformed_date_str)
                .map_err(|e| e.to_string())
                .expect_err("should error");
            assert!(
                actual_error.contains(expected_error),
                "expected error to contain: '{}', actual: '{}'", expected_error, actual_error,
            );
        }
    }
}
