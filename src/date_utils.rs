use crate::error::TxtodoError;
use time::{Date, Month, format_description::BorrowedFormatItem, macros::format_description};

const DATE_FORMAT: &[BorrowedFormatItem<'_>] = format_description!("[year]-[month]-[day]");

#[must_use]
pub(crate) fn is_date(token: &str) -> bool {
    if token.len() != 10 {
        return false;
    }
    let bytes = token.as_bytes();
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit())
}

pub(crate) fn parse_date(date_str: &str) -> Result<Date, TxtodoError> {
    if !is_date(date_str) {
        return Err(TxtodoError::Date {
            message: format!("Invalid date format: {date_str}"),
            date_str: Some(date_str.to_string()),
        });
    }
    let year: i32 = date_str[0..4].parse().unwrap();
    let month_num: u8 = date_str[5..7].parse().unwrap();
    let day: u8 = date_str[8..10].parse().unwrap();

    let month = Month::try_from(month_num).map_err(|_| TxtodoError::Date {
        message: format!("Invalid month: {month_num}"),
        date_str: Some(date_str.to_string()),
    })?;

    Date::from_calendar_date(year, month, day).map_err(|e| TxtodoError::Date {
        message: format!("Invalid date: {e}"),
        date_str: Some(date_str.to_string()),
    })
}

#[must_use]
pub(crate) fn format_date(date: Date) -> String {
    date.format(DATE_FORMAT)
        .unwrap_or_else(|_| String::from("invalid"))
}
