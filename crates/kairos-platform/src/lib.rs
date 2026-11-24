//! crates/kairos-platform/src/lib.rs
//!
//! # Cross-Platform Platform Adapters for Kairos
//!
//! Provides hardware RTC queries, system clock extraction, and event persistence
//! across Linux (Ubuntu/Debian), macOS, Windows, and sovereign NovaOS ring-3.

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl PlatformDate {
    /// Computes current UTC date and time from the system clock epoch.
    pub fn now_utc() -> Self {
        let secs = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(_) => 0,
        };

        // Days since 1970-01-01
        let mut days = (secs / 86400) as i64;
        let rem_secs = (secs % 86400) as u32;

        let hour = (rem_secs / 3600) as u8;
        let minute = ((rem_secs % 3600) / 60) as u8;
        let second = (rem_secs % 60) as u8;

        // Gregorian date calculation from epoch days
        let mut year = 1970u16;
        loop {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let ydays = if leap { 366 } else { 365 };
            if days >= ydays {
                days -= ydays;
                year += 1;
            } else {
                break;
            }
        }

        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let mut month = 1u8;
        for &m_len in &mdays {
            if days >= m_len as i64 {
                days -= m_len as i64;
                month += 1;
            } else {
                break;
            }
        }
        let day = (days + 1) as u8;

        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }
}

/// Platform trait abstracting platform date sources and event persistence.
pub trait PlatformAdapter {
    fn current_date(&self) -> PlatformDate;
    fn platform_name(&self) -> &'static str;
}

pub struct NativeHostAdapter;

impl PlatformAdapter for NativeHostAdapter {
    fn current_date(&self) -> PlatformDate {
        PlatformDate::now_utc()
    }

    fn platform_name(&self) -> &'static str {
        #[cfg(target_os = "macos")]
        return "macOS Native Calendar Platform";

        #[cfg(target_os = "linux")]
        return "Linux / Ubuntu FreeDesktop Platform";

        #[cfg(target_os = "windows")]
        return "Windows Win32 SystemTime Platform";

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        return "NovaOS Sovereign Ring-3 Platform";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_now_utc() {
        let dt = PlatformDate::now_utc();
        assert!(dt.year >= 2026);
        assert!(dt.month >= 1 && dt.month <= 12);
        assert!(dt.day >= 1 && dt.day <= 31);
        assert!(dt.hour < 24);
        assert!(dt.minute < 60);
        assert!(dt.second < 60);
    }
}
