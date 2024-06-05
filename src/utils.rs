use chrono::NaiveDate;

/**
 * Get the last day of the year
 */
pub fn last_day_of_year(year: i32) -> NaiveDate {
    let date = NaiveDate::from_ymd_opt(year + 1, 1, 1).expect("Failed to create Date");
    date.pred_opt().unwrap()
}

/**
 * Get the last day of the month
 */
pub fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };

    date.expect("Failed to create Date").pred_opt().unwrap()
}
