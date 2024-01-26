//! Demo command line tool that uses the bank holiday library

use tracing_subscriber::{EnvFilter, filter::LevelFilter, prelude::*};

use govuk_bank_holidays::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::builder()
            .with_default_directive(LevelFilter::DEBUG.into())
            .from_env_lossy())
        .init();

    let today = Date::today();

    let mut args = std::env::args();
    let program = args.next()
        .and_then(|program| {
            program.rsplit_once('/')
                .map(|(_path, program)| program.to_owned())
        })
        .unwrap_or("bank-holidays".to_owned());

    match args.next().as_deref() {
        Some("today") | None => {
            let calendar = BankHolidayCalendar::load().await;
            let is_holiday = calendar.is_holiday(today, None);
            if is_holiday {
                println!("Today ({today}) is a bank holiday");
            } else {
                println!("Today ({today}) is not a bank holiday");
            }
        }
        Some("next") => {
            let calendar = BankHolidayCalendar::load().await;
            if let Some(holiday) = calendar.iter_holidays_after(today, None).next() {
                println!("The next bank holiday is {} ({})", holiday.title, holiday.date);
            } else {
                eprintln!("Next bank holiday cannot be determined");
                std::process::exit(2);
            };
        }
        Some("previous") => {
            let calendar = BankHolidayCalendar::load().await;
            if let Some(holiday) = calendar.iter_holidays_before(today, None).next() {
                println!("The previous bank holiday was {} ({})", holiday.title, holiday.date);
            } else {
                eprintln!("Previous bank holiday cannot be determined");
                std::process::exit(2);
            };
        }
        Some("help") | Some("--help") | Some("-h") => {
            eprintln!("Bank holiday information from GOV.UK data\nUsage:\n  {program} [today]   Prints whether today is a bank holiday\n  {program} next      Prints the next bank holiday date\n  {program} previous  Prints the previous bank holiday date\n  {program} help      Prints this help information");
        }
        _ => {
            eprintln!("Unknown argument");
            std::process::exit(1);
        }
    };
    if args.next().is_some() {
        eprintln!("Too many arguments");
        std::process::exit(1);
    }
}
