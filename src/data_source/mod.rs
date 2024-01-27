//! Utilities for load and parsing bank holidays from GOV.UK.

use std::collections::HashMap;
use std::fmt::Formatter;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de;
use serde::ser::SerializeMap;

use crate::{BankHoliday, Division, Error};

mod cached;
mod reqwest;

pub use cached::Cached;
pub use reqwest::Reqwest;

/// Represents a mapping of “divisions” to bank holidays.
/// A concrete [`BankHolidayCalendar`](crate::BankHolidayCalendar) is built from this.
/// Can be de/serialised from/to the JSON format used by [GOV.UK](https://www.gov.uk/bank-holidays.json).
pub struct DataSource {
    holiday_map: HashMap<Division, Vec<BankHoliday>>,
}

impl DataSource {
    /// Construct data source from division to bank holidays mapping.
    ///
    /// NB: Call [`DataSource::sort`] if holidays might not be in date order.
    #[inline]
    pub fn new(holiday_map: HashMap<Division, Vec<BankHoliday>>) -> Self {
        // TODO: force sort?
        Self { holiday_map }
    }

    /// Parse JSON bytes.
    pub fn try_from_json<T: AsRef<[u8]>>(json: T) -> Result<Self, Error> {
        serde_json::from_slice(json.as_ref()).map_err(Error::Parsing)
    }

    #[inline]
    pub(crate) fn into_inner(self) -> HashMap<Division, Vec<BankHoliday>> {
        self.holiday_map
    }

    /// Sort each division by date.
    pub fn sort(&mut self) {
        for events in self.holiday_map.values_mut() {
            events.sort();
        }
    }

    /// Ensure all divisions are present.
    pub fn add_missing_divisions(&mut self) {
        for division in Division::all() {
            self.holiday_map.entry(division).or_default();
        }
    }

    /// Merge with another data source, division by division,
    /// with `other` data source overriding this one if the same date appears in both.
    pub fn merge(&mut self, other: DataSource) {
        for (division, other_events) in other.holiday_map {
            if let Some(events) = self.holiday_map.get_mut(&division) {
                let mut other_events = other_events;
                let mut merged_events = Vec::with_capacity(events.len());
                {
                    let mut existing_events = events.drain(..).peekable();
                    let mut other_events = other_events.drain(..).peekable();
                    loop {
                        let Some(existing_event) = existing_events.peek() else {
                            merged_events.extend(other_events);
                            break;
                        };
                        let Some(other_event) = other_events.peek() else {
                            merged_events.extend(existing_events);
                            break;
                        };
                        if existing_event.date < other_event.date {
                            merged_events.push(existing_events.next().unwrap());
                        } else {
                            if existing_event.date == other_event.date {
                                existing_events.next().unwrap();
                            }
                            merged_events.push(other_events.next().unwrap());
                        }
                    }
                }
                std::mem::swap(&mut merged_events, events);
            } else {
                self.holiday_map.insert(division, other_events);
            }
        }
    }
}

impl Serialize for DataSource {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Representation<'a> {
            division: Division,
            events: &'a [BankHoliday],
        }

        let mut map = serializer.serialize_map(Some(self.holiday_map.len()))?;
        // serialised in order for stable output shape
        for division in Division::all() {
            if let Some(events) = self.holiday_map.get(&division) {
                map.serialize_key(&division)?;
                map.serialize_value(&Representation { division, events })?;
            }
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for DataSource {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Representation {
            division: Division,
            events: Vec<BankHoliday>,
        }

        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = HashMap<Division, Representation>;

            fn expecting(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a map of divisions to division and events lists")
            }

            fn visit_map<M: de::MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut data = HashMap::with_capacity(3);
                while let Some((division, events)) = map.next_entry()? {
                    data.insert(division, events);
                }
                Ok(data)
            }
        }

        let mut data = deserializer.deserialize_map(Visitor)?;
        let mut holiday_map = HashMap::with_capacity(data.len());
        for (division, repr) in data.drain() {
            if division != repr.division {
                return Err(de::Error::custom("divisions do not match"));
            }
            holiday_map.insert(division, repr.events);
        }
        Ok(DataSource { holiday_map })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BankHolidayCalendar, Date, Weekday};

    #[test]
    fn manual_data_source() {
        let new_year_2024_to_2030: Vec<_> = (2024..2031)
            .map(|year| {
                let mut holiday = Date::try_from_components(year, 1, 1).unwrap();
                let mut notes = "";
                while matches!(holiday.weekday(), Weekday::Saturday | Weekday::Sunday) {
                    holiday = holiday.next_day();
                    notes = "Substitute day";
                }
                BankHoliday {
                    date: holiday,
                    title: "New Year’s Day".to_owned(),
                    notes: notes.to_owned(),
                    bunting: false,
                }
            })
            .collect();
        let holiday_map: HashMap<_, _> = Division::all()
            .into_iter()
            .map(move |division| {
                (division, new_year_2024_to_2030.clone())
            })
            .collect();
        let data_source = DataSource::new(holiday_map);
        let calendar = BankHolidayCalendar::new(data_source);
        let holidays = calendar.holidays(None);
        assert_eq!(holidays.len(), 2031 - 2024);
        assert!(holidays.iter().all(|holiday| {
            holiday.as_ref().month() == 1
                && holiday.title.as_str() == "New Year’s Day"
                && holiday.notes.as_str() == if holiday.date.day() > 1 { "Substitute day" } else { "" }
        }));
    }
}
