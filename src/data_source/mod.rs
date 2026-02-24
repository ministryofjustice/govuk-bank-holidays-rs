//! Utilities for load and parsing bank holidays from GOV.UK.

use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::{de, de::MapAccess, Deserialize, Deserializer, ser::SerializeMap, Serialize, Serializer};

use crate::{BankHoliday, Division, Error, PlainDate};

mod cached;
mod reqwest;

pub use cached::Cached;
pub use reqwest::Reqwest;

/// A trait to allow clients of the library to load bank holidays by other means.
pub trait LoadDataSource<Date: PlainDate> {
    /// Load a [`DataSource`] or return an [`Error`].
    #[allow(async_fn_in_trait)]
    // #[expect(async_fn_in_trait, reason = "auto trait bounds do not need specifying inside this library")]
    async fn load_data_source(&self) -> Result<DataSource<Date>, Error>;
}

/// Represents a mapping of “divisions” to bank holidays.
/// A concrete [`BankHolidayCalendar`](crate::BankHolidayCalendar) is built from this.
/// Can be de/serialised from/to the JSON format used by [GOV.UK](https://www.gov.uk/bank-holidays.json).
pub struct DataSource<Date: PlainDate> {
    holiday_map: HashMap<Division, Vec<BankHoliday<Date>>>,
}

impl<Date: PlainDate> DataSource<Date> {
    /// Construct data source from division to bank holidays mapping.
    ///
    /// NB: Call [`DataSource::sort`] if holidays might not be in date order.
    #[inline]
    pub fn new(holiday_map: HashMap<Division, Vec<BankHoliday<Date>>>) -> Self {
        Self { holiday_map }
    }

    /// Parse JSON bytes.
    pub fn try_from_json<T: AsRef<[u8]>>(json: T) -> Result<Self, Error> {
        serde_json::from_slice(json.as_ref()).map_err(Error::Parsing)
    }

    #[inline]
    pub(crate) fn into_inner(self) -> HashMap<Division, Vec<BankHoliday<Date>>> {
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
    /// Both are assumed to be sorted already.
    pub fn merge(&mut self, other: DataSource<Date>) {
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
                        if existing_event.date() < other_event.date() {
                            merged_events.push(existing_events.next().unwrap());
                        } else {
                            if existing_event.date() == other_event.date() {
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

impl<Date: PlainDate> Serialize for DataSource<Date> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Representation<'a, Date: PlainDate> {
            division: Division,
            events: &'a [BankHoliday<Date>],
        }

        impl<'a, Date: PlainDate> Serialize for Representation<'a, Date> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("division", &self.division)?;
                map.serialize_entry("events", &self.events)?;
                map.end()
            }
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

impl<'de, Date: PlainDate> Deserialize<'de> for DataSource<Date> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Visitor<Date>(PhantomData<fn() -> Date>);

        impl<'de, Date: PlainDate> de::Visitor<'de> for Visitor<Date> {
            type Value = DataSource<Date>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map of divisions to division and events lists")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                #[derive(Deserialize)]
                struct Representation<Date: PlainDate> {
                    division: Division,
                    #[serde(deserialize_with = "Vec::deserialize")]
                    events: Vec<BankHoliday<Date>>,
                }

                let mut data: HashMap<Division, Representation<Date>> = HashMap::with_capacity(3);
                while let Some((division, events)) = map.next_entry()? {
                    data.insert(division, events);
                }
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

        deserializer.deserialize_map(Visitor(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BankHolidayCalendar, BankHoliday, Weekday};

    fn check_data_source<Date: PlainDate>() {
        let mut data_source: DataSource<Date> = DataSource::new(HashMap::new());
        data_source.add_missing_divisions();
        assert!(Division::all().iter().all(|division| data_source.holiday_map.contains_key(division)));

        // language=json
        let source1 = r#"{
          "scotland": {
            "division": "scotland",
            "events": [
              {
                "date": "2022-12-26",
                "title": "Boxing Day",
                "notes": "",
                "bunting": true
              },
              {
                "date": "2022-11-30",
                "title": "St Andrew’s Day",
                "notes": "from source 1",
                "bunting": true
              }
            ]
          }
        }"#;
        let mut source1 = DataSource::<Date>::try_from_json(source1)
            .expect("data source should be valid");
        // language=json
        let source2 = r#"{
          "scotland": {
            "division": "scotland",
            "events": [
              {
                "date": "2022-11-30",
                "title": "St Andrew’s Day",
                "notes": "from source 2",
                "bunting": true
              }
            ]
          }
        }"#;
        let source2 = DataSource::<Date>::try_from_json(source2)
            .expect("data source should be valid");
        source1.sort();
        source1.merge(source2);
        let source = source1.into_inner();
        let bank_holidays = &source[&Division::Scotland];
        assert_eq!(bank_holidays.len(), 2);
        assert_eq!(bank_holidays[0].notes(), "from source 2");

        // language=json
        let source3 = r#"{
          "scotland": {
            "division": "england-and-wales",
            "events": [
              {
                "date": "2022-11-30",
                "title": "St Andrew’s Day",
                "notes": "from source 2",
                "bunting": true
              }
            ]
          }
        }"#;
        assert!(DataSource::<Date>::try_from_json(source3).is_err());
    }

    #[cfg(feature = "chrono")]
    #[test]
    fn chrono_data_source() {
        check_data_source::<crate::dates::chrono::DateImpl>();
    }

    #[cfg(feature = "time")]
    #[test]
    fn time_data_source() {
        check_data_source::<crate::dates::time::DateImpl>();
    }

    #[cfg(any(feature = "chrono", feature = "time"))]
    async fn custom_loader<Date: PlainDate>() {
        struct NewYearsDays<Date: PlainDate>(PhantomData<Date>);

        impl<Date: PlainDate> LoadDataSource<Date> for NewYearsDays<Date> {
            async fn load_data_source(&self) -> Result<DataSource<Date>, Error> {
                let new_year_2024_to_2030: Vec<_> = (2024..2031)
                    .map(|year| {
                        let mut holiday = Date::try_from_components(year, 1, 1).unwrap();
                        let mut notes = "";
                        while matches!(holiday.weekday(), Weekday::Saturday | Weekday::Sunday) {
                            holiday = holiday.next_day();
                            notes = "Substitute day";
                        }
                        BankHoliday::new_with_notes(holiday, "New Year’s Day".to_owned(), notes.to_owned())
                    })
                    .collect();
                let holiday_map: HashMap<_, _> = Division::all()
                    .into_iter()
                    .map(move |division| {
                        (division, new_year_2024_to_2030.clone())
                    })
                    .collect();
                let data_source = DataSource::new(holiday_map);
                Ok(data_source)
            }
        }

        let calendar = BankHolidayCalendar::custom(NewYearsDays(PhantomData::<Date>)).await
            .expect("calendar should load");
        let holidays = calendar.holidays(None);
        assert_eq!(holidays.len(), 2031 - 2024);
        assert!(holidays.iter().all(|holiday| {
            holiday.as_ref().month() == 1
                && holiday.title() == "New Year’s Day"
                && holiday.notes() == if holiday.date().day() > 1 { "Substitute day" } else { "" }
        }));
    }

    #[cfg(feature = "chrono")]
    #[tokio::test]
    async fn chrono_custom_loader() {
        custom_loader::<crate::dates::chrono::DateImpl>().await;
    }

    #[cfg(feature = "time")]
    #[tokio::test]
    async fn time_custom_loader() {
        custom_loader::<crate::dates::time::DateImpl>().await;
    }
}
