GOV.UK Bank Holidays
====================

This [rust](https://www.rust-lang.org/) library loads the official list of bank holidays in the United Kingdom
as supplied by [GOV.UK](https://www.gov.uk/bank-holidays),
which tends to provide this list for only a year or two into the future.

A cached backup list of known bank holidays is stored in this package, though it is not updated often.
GOV.UK no longer provide bank holidays for some of the older years still part of this backup list.

Bank holidays differ around the UK. The GOV.UK source currently lists these for 3 “divisions”:

- England and Wales
- Scotland
- Northern Ireland

Methods on `BankHolidayCalendar` that take a `division` parameter will consider bank holidays only for the provided
division (eg. `Some(Division::Scotland)`) or only those that are **common** to all divisions for `None`.

Using the library
-----------------

[![Test, lint & publish](https://github.com/ministryofjustice/govuk-bank-holidays-rs/actions/workflows/pipeline.yml/badge.svg)](https://github.com/ministryofjustice/govuk-bank-holidays-rs/actions/workflows/pipeline.yml)
[![crates.io](https://img.shields.io/crates/v/govuk-bank-holidays)](http://crates.io/crates/govuk-bank-holidays)
[![docs.rs](https://img.shields.io/docsrs/govuk-bank-holidays)](https://docs.rs/govuk-bank-holidays)

Add to your project with:

```shell
cargo add govuk-bank-holidays
```

See [docs.rs](https://docs.rs/govuk-bank-holidays) for API information, usage samples and feature flags.

Developing library
------------------

### Dependencies

- rust 1.75+ (using [rustup](https://rustup.rs/) is recommended)
- [just](https://just.systems/man/en/) (optional) – for scripted shortcuts, akin to `make`

### Making changes

Run unit tests with `just test`.
Update cached bank holidays from GOV.UK using `just refresh-cache`.

See `just` for other scripted shortcuts.

### Releasing a new version

- Update version in `Cargo.toml`
- Add to [History](#history) with changes since last release
- Tag and [publish a release on GitHub](https://github.com/ministryofjustice/govuk-bank-holidays-rs/releases)
  which triggers publishing to [crates.io](http://crates.io/crates/govuk-bank-holidays)

TODO
----

- Optionally merge in older known bank holidays into newly-downloaded GOV.UK data? Cached data starts in 2012,
  but currently GOV.UK provides nothing before 2024.
- Better tests, coverage
- Performance improvements (particularly around memory and iterators)
- Loading data:
  - Can `DataSource` be made private, exposing methods on `LoadDataSource` trait or elsewhere?
  - Make `reqwest` an optional feature? This might allow for no-std calendar of baked-in bank holidays.
- Divisions:
  - Allow unifying all divisions such that all bank holiday are returned labelled with where they apply?
  - Allow for unknown divisions? Make `Division` enum non-exhaustive?

References
----------

See also:

- [GOV.UK bank holidays](https://www.gov.uk/bank-holidays)
- [Python library](https://github.com/ministryofjustice/govuk-bank-holidays)

History
-------

### 0.3.0
**Major breaking changes!**
Bring-your-own date library: the implementation is now customisable
and both `chrono` and `time` features can be used together.
The api has received many changes, but client code should need relatively minor adaptation.

Notable differences include:
- The library is now generic over the date implementation using a new `PlainDate` trait.
  It’s implemented for `chrono::NaiveDate` and `time::Date`, but consumers are able to use their own.
- The date implementation is no longer expected to be `Copy`, `Display`, `serde::Deserialize` nor `serde::Serialize`.
  A  simple internal serde implementation exists to work consistently for all date types.
- Many `BankHolidayCalendar` methods now take borrowed dates, instead of owned ones.
- `BankHoliday` is now immutable.
- `Cached::cached_data_source` is now a method needing a default `Cached` instance.

### 0.2.3
Updated cached bank holidays and dependencies.

### 0.2.2
Updated cached bank holidays.

### 0.2.1
Updated dependencies.

### 0.2.0
Allow for custom sources of bank holidays using `LoadDataSource` trait.

### 0.1.1
Very minor changes – tidier code and improved documentation.

### 0.1.0
Initial release with API likely to be unstable.
