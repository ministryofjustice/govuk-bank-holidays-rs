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
division (`Some(Division)`) or only those that are **common** to all divisions for `None`.

Using the library
-----------------

![github](https://github.com/ministryofjustice/govuk-bank-holidays-rs/actions/workflows/pipeline.yml/badge.svg?branch=main)
![crates.io](https://img.shields.io/crates/v/govuk-bank-holidays)
![docs.rs](https://img.shields.io/docsrs/govuk-bank-holidays)

Add to your project with:

```shell
cargo add govuk-bank-holidays
```

See [docs.rs](https://docs.rs/govuk-bank-holidays) for API information, usage samples and feature flags.

Developing library
------------------

### Requirements

- rust 1.75+ (using [rustup](https://rustup.rs/) is recommended)
- [just](https://just.systems/man/en/) – for scripted shortcuts; like `make`

### Releasing a new version

- Update version in `Cargo.toml`
- Add to [History](#history) with changes since last release
- Tag and [publish a release on GitHub](https://github.com/ministryofjustice/govuk-bank-holidays-rs/releases)
  which triggers publishing to [crates.io](http://crates.io/crates/govuk-bank-holidays)

TODO
----

- Better tests, coverage
- Optionally merge in older known bank holidays into newly-downloaded GOV.UK data? Cached data starts in 2012,
  but currently GOV.UK provides nothing before 2018.
- Performance improvements (particularly around memory and iterators)
- Can `DataSource` be made private, exposing methods on `LoadDataSource` trait or elsewhere?
- Allow for unknown “divisions”? Make enum non-exhaustive?

References
----------

See also:

- [GOV.UK bank holidays](https://www.gov.uk/bank-holidays)
- [Python library](https://github.com/ministryofjustice/govuk-bank-holidays)

History
-------

### 0.2.1
Updated dependencies

### 0.2.0
Allow for custom sources of bank holidays using `LoadDataSource` trait.

### 0.1.1
Very minor changes – tidier code and improved documentation.

### 0.1.0
Initial release with API likely to be unstable.
