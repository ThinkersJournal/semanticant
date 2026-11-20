//! Values from text: numbers, quantities with units, dates and times. See the crate README.
#![forbid(unsafe_code)]

mod dates;
mod error;
mod numbers;
mod units;

pub use dates::{CalendarRules, DateMatch, PartialDate, TimeOfDay, parse_dates, parse_times};
pub use error::ValueError;
pub use numbers::{Found, NumberRules, WordKind, parse_numbers};
pub use rust_decimal::Decimal;
pub use units::{ConvertError, Quantity, UnitDef, UnitTable, convert, parse_quantities};
