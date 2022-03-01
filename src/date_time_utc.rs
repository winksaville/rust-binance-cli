use std::fmt::Display;

use chrono::{prelude::*, Datelike, Utc};

use crate::common::time_ms_to_utc;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DateTimeUtc {
    dt: DateTime<Utc>,
}

impl DateTimeUtc {
    pub fn new(date: &DateTime<Utc>) -> DateTimeUtc {
        DateTimeUtc { dt: *date }
    }

    pub fn from_utc(date: DateTime<Utc>) -> DateTimeUtc {
        DateTimeUtc { dt: date }
    }

    pub fn from_utc_time_ms(utc_time_ms: i64) -> DateTimeUtc {
        DateTimeUtc::new(&time_ms_to_utc(utc_time_ms))
    }

    pub fn from_utc_ymd_hmsn(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        nano: u32,
    ) -> DateTimeUtc {
        DateTimeUtc::from_utc(Utc.ymd(year, month, day).and_hms_nano(hour, min, sec, nano))
    }

    pub fn get_dt(&self) -> DateTime<Utc> {
        self.dt
    }

    #[allow(unused)]
    pub fn days_in_month(&self) -> u32 {
        let (year, month, _, _, _, _, _) = self.parts();
        Self::days_in_month_from_year_month(year, month)
    }

    pub fn beginning_of_this_month(&self) -> DateTimeUtc {
        let (year, month, _, _, _, _, _) = self.parts();
        DateTimeUtc::from_utc(Utc.ymd(year, month, 1).and_hms_nano(0, 0, 0, 0))
    }

    #[allow(unused)]
    pub fn beginning_of_this_day(&self) -> DateTimeUtc {
        let (year, month, day, _, _, _, _) = self.parts();
        DateTimeUtc::from_utc(Utc.ymd(year, month, day).and_hms_nano(0, 0, 0, 0))
    }

    pub fn beginning_of_next_month(&self) -> DateTimeUtc {
        let (y, m, _, _, _, _, _) = self.parts();
        let (y, m) = Self::calculate_next_month(y, m);

        DateTimeUtc::from_utc_ymd_hmsn(y, m, 1, 0, 0, 0, 0)
    }

    #[allow(unused)]
    pub fn beginning_of_next_day(&self) -> DateTimeUtc {
        let (year, month, day, _, _, _, _) = self.parts();
        let (year, month, day) = Self::calculate_next_day(year, month, day);

        DateTimeUtc::from_utc_ymd_hmsn(year, month, day, 0, 0, 0, 0)
    }

    #[allow(unused)]
    pub fn signed_duration_since_in_secs(&self, rhs: &DateTimeUtc) -> i64 {
        self.get_dt()
            .signed_duration_since(rhs.get_dt())
            .num_seconds()
    }

    #[inline(always)]
    pub fn time_nanos(&self) -> i64 {
        self.get_dt().timestamp_nanos()
    }

    #[inline(always)]
    pub fn time_ms(&self) -> i64 {
        (self.time_nanos() + 500_000) / 1_000_000
    }

    #[inline(always)]
    pub fn year(&self) -> i32 {
        self.get_dt().year()
    }

    #[inline(always)]
    pub fn month(&self) -> u32 {
        self.get_dt().month()
    }

    #[inline(always)]
    pub fn day(&self) -> u32 {
        self.get_dt().day()
    }

    #[inline(always)]
    pub fn hour(&self) -> u32 {
        self.get_dt().hour()
    }

    #[inline(always)]
    pub fn minute(&self) -> u32 {
        self.get_dt().minute()
    }

    #[inline(always)]
    pub fn second(&self) -> u32 {
        self.get_dt().second()
    }

    #[inline(always)]
    pub fn nanosecond(&self) -> u32 {
        self.get_dt().nanosecond()
    }

    #[inline(always)]
    pub fn parts(&self) -> (i32, u32, u32, u32, u32, u32, u32) {
        (
            self.year(),
            self.month(),
            self.day(),
            self.hour(),
            self.minute(),
            self.second(),
            self.nanosecond(),
        )
    }

    fn days_in_month_from_year_month(y_today: i32, m_today: u32) -> u32 {
        let (y_tomorrow, m_tomorrow) = Self::calculate_next_month(y_today, m_today);
        Utc.ymd(y_tomorrow, m_tomorrow, 1)
            .signed_duration_since(Utc.ymd(y_today, m_today, 1))
            .num_days() as u32
    }

    fn calculate_next_month(year: i32, month: u32) -> (i32, u32) {
        let mut m = month + 1;
        let mut y = year;
        if m > 12 {
            y += 1;
            m = 1;
        }
        (y, m)
    }

    #[allow(unused)]
    fn calculate_next_day(year: i32, month: u32, day: u32) -> (i32, u32, u32) {
        let mut y_tomorrow = year;
        let mut m_tomorrow = month;
        let mut d_tomorrow = day + 1;
        if d_tomorrow > Self::days_in_month_from_year_month(year, month) {
            d_tomorrow = 1;
            let (y, m) = Self::calculate_next_month(year, month);
            y_tomorrow = y;
            m_tomorrow = m;
        }

        (y_tomorrow, m_tomorrow, d_tomorrow)
    }
}

impl Display for DateTimeUtc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.get_dt())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let utc_now = Utc::now();
        let dt_utc = DateTimeUtc::new(&utc_now);
        assert_eq!(utc_now, dt_utc.get_dt())
    }

    #[test]
    fn test_from_utc_time_ms() {
        let dtu = DateTimeUtc::from_utc_time_ms(0);
        let (year, month, day, hour, min, sec, nano) = dtu.parts();
        assert_eq!(year, 1970);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);
    }

    #[test]
    fn test_get_dt() {
        let utc_now = Utc::now();
        let dt_utc = DateTimeUtc::new(&utc_now);
        assert_eq!(utc_now, dt_utc.get_dt())
    }

    #[test]
    fn test_parts() {
        let utc_now = Utc::now();
        let dt_utc = DateTimeUtc::new(&utc_now);
        let (year, month, day, hour, min, sec, nano) = dt_utc.parts();
        assert_eq!(year, utc_now.year());
        assert_eq!(month, utc_now.month());
        assert_eq!(day, utc_now.day());
        assert_eq!(hour, utc_now.hour());
        assert_eq!(min, utc_now.minute());
        assert_eq!(sec, utc_now.second());
        assert_eq!(nano, utc_now.nanosecond());
    }

    #[test]
    fn test_beginning_of_this_month() {
        let lt_str = "2022-02-26T14:23:43.123+00:00";
        let dt: DateTime<Utc> = lt_str.parse().unwrap();
        let dt_utc = DateTimeUtc::new(&dt);
        let (year, month, day, hour, min, sec, nano) = dt_utc.parts();
        assert_eq!(year, 2022);
        assert_eq!(month, 2);
        assert_eq!(day, 26);
        assert_eq!(hour, 14);
        assert_eq!(min, 23);
        assert_eq!(sec, 43);
        assert_eq!(nano, 123 * 1_000_000);

        let start_of_month = dt_utc.beginning_of_this_month();
        let (year, month, day, hour, min, sec, nano) = start_of_month.parts();
        assert_eq!(year, 2022);
        assert_eq!(month, 2);
        assert_eq!(day, 1);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);
    }

    #[test]
    fn test_beginning_of_next_month() {
        let lt_str = "2022-01-01T00:00:00+00:00";
        let dt: DateTime<Utc> = lt_str.parse().unwrap();
        let dt_utc = DateTimeUtc::new(&dt);
        let next_month = dt_utc.beginning_of_next_month();

        let (year, month, day, hour, min, sec, nano) = next_month.parts();
        assert_eq!(year, 2022);
        assert_eq!(month, 2);
        assert_eq!(day, 1);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);

        let lt_str = "2022-12-12T00:00:00+00:00";
        let dt: DateTime<Utc> = lt_str.parse().unwrap();
        let dt_utc = DateTimeUtc::new(&dt);
        let next_month = dt_utc.beginning_of_next_month();

        let (year, month, day, hour, min, sec, nano) = next_month.parts();
        assert_eq!(year, 2023);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);
    }

    #[test]
    fn test_beginning_of_today() {
        let lt_str = "2022-02-26T14:23:43.123+00:00";
        let dt: DateTime<Utc> = lt_str.parse().unwrap();
        let dt_utc = DateTimeUtc::new(&dt);
        let (year, month, day, hour, min, sec, nano) = dt_utc.parts();
        assert_eq!(year, 2022);
        assert_eq!(month, 2);
        assert_eq!(day, 26);
        assert_eq!(hour, 14);
        assert_eq!(min, 23);
        assert_eq!(sec, 43);
        assert_eq!(nano, 123 * 1_000_000);

        let start_of_month = dt_utc.beginning_of_this_day();
        let (year, month, day, hour, min, sec, nano) = start_of_month.parts();
        assert_eq!(year, 2022);
        assert_eq!(month, 2);
        assert_eq!(day, 26);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);
    }

    #[test]
    fn test_beginning_of_tomorrow() {
        let lt_str = "2022-01-01T02:03:04.123+00:00";
        let dt: DateTime<Utc> = lt_str.parse().unwrap();
        let dt_utc = DateTimeUtc::new(&dt);
        let tomorrow = dt_utc.beginning_of_next_day();

        let (year, month, day, hour, min, sec, nano) = tomorrow.parts();
        assert_eq!(year, 2022);
        assert_eq!(month, 1);
        assert_eq!(day, 2);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);

        let lt_str = "2022-12-31T02:03:04.123+00:00";
        let dt: DateTime<Utc> = lt_str.parse().unwrap();
        let dt_utc = DateTimeUtc::new(&dt);
        let next_month = dt_utc.beginning_of_next_day();

        let (year, month, day, hour, min, sec, nano) = next_month.parts();
        assert_eq!(year, 2023);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);
    }

    #[test]
    fn test_days_in_year_month() {
        //leap second
        let days = DateTimeUtc::days_in_month_from_year_month(2016, 12);
        assert_eq!(days, 31);

        let days = DateTimeUtc::days_in_month_from_year_month(2020, 1);
        assert_eq!(days, 31);

        let days = DateTimeUtc::days_in_month_from_year_month(2020, 2);
        assert_eq!(days, 29);

        let days = DateTimeUtc::days_in_month_from_year_month(2022, 2);
        assert_eq!(days, 28);
    }

    #[test]
    fn test_signed_duration_since_in_secs() {
        const TYPICAL_SECS_PER_DAY: i64 = 60 * 60 * 24;
        let normal_day = DateTimeUtc::from_utc_ymd_hmsn(2022, 02, 26, 1, 2, 3, 4);
        let normal_day = normal_day.beginning_of_this_day();
        let next_day = normal_day.beginning_of_next_day();
        let duration = next_day.signed_duration_since_in_secs(&normal_day);
        assert_eq!(duration, TYPICAL_SECS_PER_DAY);

        //leap second
        let leap_sec_day = DateTimeUtc::from_utc_ymd_hmsn(2016, 12, 31, 1, 2, 3, 4);
        let leap_sec_day = leap_sec_day.beginning_of_this_day();
        let (year, month, day, hour, min, sec, nano) = leap_sec_day.parts();
        assert_eq!(year, 2016);
        assert_eq!(month, 12);
        assert_eq!(day, 31);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);
        let next_day = leap_sec_day.beginning_of_next_day();
        let (year, month, day, hour, min, sec, nano) = next_day.parts();
        assert_eq!(year, 2017);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(hour, 0);
        assert_eq!(min, 0);
        assert_eq!(sec, 0);
        assert_eq!(nano, 0);

        let duration = next_day.signed_duration_since_in_secs(&leap_sec_day);
        println!("leap second day duration: {duration}");
        assert_ne!(duration, TYPICAL_SECS_PER_DAY + 1); // This should fail, but chrono does not handle leap seconds
        assert_eq!(duration, TYPICAL_SECS_PER_DAY);
    }
}
