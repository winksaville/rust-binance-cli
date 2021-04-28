use chrono::prelude::{DateTime, NaiveDateTime, Utc};

fn timestamp_ms_to_secs_nsecs(timestamp_ms: i64) -> (i64, u32) {
    // println!("time_ms_to_utc: + timestamp_ms={}", timestamp_ms);
    let mut secs = timestamp_ms / 1000;
    let ms: u32 = if timestamp_ms < 0 {
        // When time is less than zero the it's only negative
        // to the "epoch" thus seconds are "negative" but the
        // milli-seconds are positive. Thus -1ms is represented
        // in time as -1sec + 0.999ms. Sooooooo

        // First negate then modulo 1000 to get millis as a u32
        let mut millis = (-timestamp_ms % 1_000) as u32;

        // This is very "likely" and it would be nice to be able
        // to tell the compiler with `if likely(millis > 0) {...}
        if millis > 0 {
            // We need to reduce secs by 1
            secs -= 1;

            // And map ms 1..999 to 999..1
            millis = 1_000 - millis;
            // println!("time_ms_to_utc: adjusted   timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);
        } else {
            // millis is 0 and secs is correct as is.
            // println!("time_ms_to_utc: unadjusted timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);
        }

        millis
    } else {
        // This actually caused clippy to output "unnecessarary `let` binding"
        // but for I want to be able to have the pritnln and I've found that
        // allowing unnecessary_cast suppresses the warning.
        #[allow(clippy::unnecessary_cast)]
        let millis = (timestamp_ms % 1000) as u32;
        //println!("time_ms_to_utc: unadjusted timestamp_ms={} secs={} millis={}", timestamp_ms, secs, millis);

        millis
    };

    let nsecs = ms * 1_000_000u32;

    // println!("time_ms_to_utc: - timestamp_ms={} secs={} nsecs={}", timestamp_ms, secs, nsecs);
    (secs, nsecs)
}

pub fn time_ms_to_utc(timestamp_ms: i64) -> DateTime<Utc> {
    let (secs, nsecs) = timestamp_ms_to_secs_nsecs(timestamp_ms);
    let naive_datetime = NaiveDateTime::from_timestamp(secs, nsecs);
    DateTime::from_utc(naive_datetime, Utc)
}

pub fn utc_now_to_time_ms() -> i64 {
    (Utc::now().timestamp_nanos() + 500_000) / 1_000_000
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_timestamp_ms_to_secs_nsecs() {
        assert_eq!(timestamp_ms_to_secs_nsecs(-2001), (-3i64, 999_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(-2000), (-2i64, 0u32));
        //assert_eq!(timestamp_ms_to_secs_nsecs(-2000), (-3i64, 1_000_000_000u32)); // No Adjustment
        assert_eq!(timestamp_ms_to_secs_nsecs(-1999), (-2i64, 1_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(-1001), (-2i64, 999_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(-1000), (-1i64, 0u32));
        //assert_eq!(timestamp_ms_to_secs_nsecs(-1000), (0i64, 1_000_000_000u32)); // No adjustment
        assert_eq!(timestamp_ms_to_secs_nsecs(-999), (-1i64, 1_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(-1), (-1i64, 999_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(0), (0i64, 0u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(1), (0i64, 1_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(999), (0i64, 999_000_000u32));
        assert_eq!(timestamp_ms_to_secs_nsecs(1000), (1i64, 0u32));
    }

    #[test]
    fn test_utc_now_to_time_ms() {
        let start = Instant::now();

        // Because we use integer arithmetic we must
        // see 2 milli-second time ticks to see a minimum
        // duration of > 1ms.
        let tms1 = utc_now_to_time_ms();
        let mut tms2 = tms1;
        while tms2 < (tms1 + 2) {
            tms2 = utc_now_to_time_ms();
        }
        let done = Instant::now();
        let duration = done.duration_since(start);

        // println!(
        //     "tms1: {} tms2: {} done: {:?} - start {:?} = {}ns or {}ms",
        //     tms1,
        //     tms2,
        //     done,
        //     start,
        //     duration.as_nanos(),
        //     duration.as_millis()
        // );

        assert!(tms2 >= (tms1 + 2));
        assert!(duration.as_millis() >= 1);

        // Usually duration.as_millis will be < 2ms. But with Tarpaulin
        // I've seen durations just over 2m so assert <= 2.
        assert!(duration.as_millis() <= 2);
    }
}
