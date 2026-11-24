//! crates/kairos-core/src/lib.rs
//!
//! # Kairos Core Date, Event Scheduling & Recurrence Engine
//!
//! "Chronos measures the quantitative passage of seconds and hours;
//!  Kairos captures the opportune, appointed moment for life and events."
//!
//! Synthesizes the elegance of Apple Calendar (macOS), the productivity of
//! Windows 11 Focus Calendar, and the clean structure of GNOME Calendar (Ubuntu).

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

// ============================================================================
// Date & Time Constants
// ============================================================================

pub const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

pub const MONTH_NAMES_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub const DAY_NAMES: [&str; 7] = [
    "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
];

pub const DAY_NAMES_SHORT: [&str; 7] = [
    "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat",
];

// ============================================================================
// Gregorian Calendar Calculations
// ============================================================================

/// Returns true if the year is a leap year in the Gregorian calendar.
#[inline]
pub fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in the specified month (1..=12) for the given year.
pub fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 => 31,
        2 => if is_leap_year(year) { 29 } else { 28 },
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 30,
    }
}

/// Computes the day of the week for a given Gregorian date using Tomohiko Sakamoto's algorithm.
/// Returns 0 for Sunday, 1 for Monday, ..., 6 for Saturday.
pub fn day_of_week(year: u16, month: u8, day: u8) -> u8 {
    static T: [u16; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y = y.saturating_sub(1);
    }
    let m_idx = (month.clamp(1, 12) - 1) as usize;
    let dow = (y + y / 4 - y / 100 + y / 400 + T[m_idx] + day as u16) % 7;
    dow as u8
}

/// Formats a date into a clean string representation: "Thursday, Nov 19, 2026".
pub fn format_date_long(year: u16, month: u8, day: u8) -> String {
    let dow = day_of_week(year, month, day);
    let dow_name = DAY_NAMES[(dow % 7) as usize];
    let m_name = MONTH_NAMES_SHORT[((month.clamp(1, 12) - 1) % 12) as usize];
    format!("{}, {} {}, {}", dow_name, m_name, day, year)
}

// ============================================================================
// Event Categories & Themes
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventCategory {
    Work,       // Blue (macOS Work calendar)
    Personal,   // Green (macOS Personal / Family)
    NovaOS,     // Electric Cyan (NovaOS Milestones & Kernel Releases)
    Holiday,    // Amber (Public Holidays & Observances)
    Reminder,   // Violet (Tasks & Reminders)
    Health,     // Rose (Fitness & Wellness)
    Finance,    // Emerald (Budget & Billing)
}

impl EventCategory {
    pub fn all() -> [EventCategory; 7] {
        [
            EventCategory::Work,
            EventCategory::Personal,
            EventCategory::NovaOS,
            EventCategory::Holiday,
            EventCategory::Reminder,
            EventCategory::Health,
            EventCategory::Finance,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            EventCategory::Work => "Work & Meetings",
            EventCategory::Personal => "Personal & Life",
            EventCategory::NovaOS => "NovaOS Releases",
            EventCategory::Holiday => "Holidays",
            EventCategory::Reminder => "Tasks & Reminders",
            EventCategory::Health => "Health & Wellness",
            EventCategory::Finance => "Finance & Billing",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            EventCategory::Work => "💼",
            EventCategory::Personal => "🏠",
            EventCategory::NovaOS => "⭐",
            EventCategory::Holiday => "🎉",
            EventCategory::Reminder => "✅",
            EventCategory::Health => "❤️",
            EventCategory::Finance => "💳",
        }
    }

    pub fn color_rgb(&self) -> (u8, u8, u8) {
        match self {
            EventCategory::Work => (59, 130, 246),      // Royal Blue #3B82F6
            EventCategory::Personal => (34, 197, 94),   // Emerald Green #22C55E
            EventCategory::NovaOS => (6, 182, 212),     // Electric Cyan #06B6D4
            EventCategory::Holiday => (245, 158, 11),   // Amber Gold #F59E0B
            EventCategory::Reminder => (168, 85, 247),  // Violet Purple #A855F7
            EventCategory::Health => (244, 63, 94),     // Rose Red #F43F5E
            EventCategory::Finance => (16, 185, 129),   // Emerald #10B981
        }
    }
}

// ============================================================================
// Recurrence Rules
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecurrenceRule {
    None,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl RecurrenceRule {
    pub fn name(&self) -> &'static str {
        match self {
            RecurrenceRule::None => "Does not repeat",
            RecurrenceRule::Daily => "Every day",
            RecurrenceRule::Weekly => "Every week",
            RecurrenceRule::Monthly => "Every month",
            RecurrenceRule::Yearly => "Every year",
        }
    }
}

// ============================================================================
// View Modes
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Month,  // Full 7x5 or 7x6 month grid (Ubuntu GNOME + macOS style)
    Week,   // 7-day multi-column timeline
    Day,    // Hourly itinerary breakdown (Windows 11 schedule card)
    Agenda, // Chronological event feed (Windows 11 Focus / macOS list)
    Year,   // 12-month overview matrix
}

impl ViewMode {
    pub fn all() -> [ViewMode; 5] {
        [ViewMode::Month, ViewMode::Week, ViewMode::Day, ViewMode::Agenda, ViewMode::Year]
    }

    pub fn name(&self) -> &'static str {
        match self {
            ViewMode::Month => "Month",
            ViewMode::Week => "Week",
            ViewMode::Day => "Day",
            ViewMode::Agenda => "Agenda",
            ViewMode::Year => "Year",
        }
    }
}

// ============================================================================
// Calendar Event Data Structure
// ============================================================================

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarEvent {
    pub id: u32,
    pub title: String,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub start_time: String,
    pub end_time: String,
    pub all_day: bool,
    pub category: EventCategory,
    pub location: String,
    pub description: String,
    pub recurrence: RecurrenceRule,
    pub is_completed: bool,
}

impl CalendarEvent {
    pub fn new(
        id: u32,
        title: impl Into<String>,
        year: u16,
        month: u8,
        day: u8,
        start_time: impl Into<String>,
        end_time: impl Into<String>,
        category: EventCategory,
        location: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            year,
            month,
            day,
            start_time: start_time.into(),
            end_time: end_time.into(),
            all_day: false,
            category,
            location: location.into(),
            description: description.into(),
            recurrence: RecurrenceRule::None,
            is_completed: false,
        }
    }

    /// Checks if this event occurs on the given date (taking recurrence into account).
    pub fn occurs_on(&self, target_year: u16, target_month: u8, target_day: u8) -> bool {
        if target_year < self.year || (target_year == self.year && target_month < self.month) {
            return false;
        }

        match self.recurrence {
            RecurrenceRule::None => {
                self.year == target_year && self.month == target_month && self.day == target_day
            }
            RecurrenceRule::Daily => {
                if target_year == self.year && target_month == self.month {
                    target_day >= self.day
                } else {
                    true
                }
            }
            RecurrenceRule::Weekly => {
                let orig_dow = day_of_week(self.year, self.month, self.day);
                let target_dow = day_of_week(target_year, target_month, target_day);
                orig_dow == target_dow
            }
            RecurrenceRule::Monthly => {
                self.day == target_day
            }
            RecurrenceRule::Yearly => {
                self.month == target_month && self.day == target_day
            }
        }
    }
}

// ============================================================================
// Kairos Calendar State & Storage Engine
// ============================================================================

#[derive(Clone, Debug)]
pub struct CalendarState {
    pub now_year: u16,
    pub now_month: u8,
    pub now_day: u8,
    pub now_hour: u8,
    pub now_minute: u8,

    pub view_year: u16,
    pub view_month: u8,
    pub selected_day: u8,

    pub view_mode: ViewMode,
    pub enabled_categories: [bool; 7],

    pub events: Vec<CalendarEvent>,
    pub next_event_id: u32,
    pub selected_event_id: Option<u32>,

    pub timezone_name: String,
    pub timezone_offset_hours: i8,
}

impl CalendarState {
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        let mut s = Self {
            now_year: year,
            now_month: month,
            now_day: day,
            now_hour: 9,
            now_minute: 41,

            view_year: year,
            view_month: month,
            selected_day: day,

            view_mode: ViewMode::Month,
            enabled_categories: [true; 7],

            events: Vec::new(),
            next_event_id: 1,
            selected_event_id: None,

            timezone_name: "UTC".to_string(),
            timezone_offset_hours: 0,
        };

        s.seed_default_events();
        s
    }

    /// Seeds enterprise milestone events, holidays, and schedule templates.
    pub fn seed_default_events(&mut self) {
        let y = self.now_year;
        let m = self.now_month;

        self.add_event(CalendarEvent::new(
            0,
            "NovaOS 2.0 GA Worldwide Launch",
            y,
            m,
            19,
            "09:00 AM",
            "10:30 AM",
            EventCategory::NovaOS,
            "Main Auditorium & Virtual Keynote",
            "Official sovereign desktop release with bare-metal window manager and app store.",
        ));

        self.add_event(CalendarEvent::new(
            0,
            "Architecture Review & Kernel Benchmarks",
            y,
            m,
            19,
            "02:00 PM",
            "03:30 PM",
            EventCategory::Work,
            "Conference Room A",
            "SMP scalability, memory safety benchmarks, and virtio-gpu hardware acceleration.",
        ));

        self.add_event(CalendarEvent::new(
            0,
            "Kairos Calendar v1.0 Integration",
            y,
            m,
            24,
            "11:00 AM",
            "12:00 PM",
            EventCategory::NovaOS,
            "Sovereign Workspace",
            "Companion cross-platform repository release synthesizing macOS & Ubuntu.",
        ));

        self.add_event(CalendarEvent::new(
            0,
            "Team Retrospective & Celebration",
            y,
            m,
            24,
            "04:00 PM",
            "05:00 PM",
            EventCategory::Personal,
            "Lounge",
            "Celebrating 18/18 UI test suites passing with 100% QA success.",
        ));

        self.add_event(CalendarEvent::new(
            0,
            "Thanksgiving Day",
            y,
            11,
            26,
            "All Day",
            "All Day",
            EventCategory::Holiday,
            "Nationwide",
            "Public holiday and festival of gratitude.",
        ));

        self.add_event(CalendarEvent::new(
            0,
            "Weekly Kernel Sync",
            y,
            m,
            20,
            "10:00 AM",
            "11:00 AM",
            EventCategory::Work,
            "Virtual / Terminal",
            "Recurring Friday kernel engineering sync.",
        ));

        if let Some(ev) = self.events.last_mut() {
            ev.recurrence = RecurrenceRule::Weekly;
        }

        self.add_event(CalendarEvent::new(
            0,
            "Pay Cloud Infrastructure Invoice",
            y,
            m,
            30,
            "09:00 AM",
            "09:30 AM",
            EventCategory::Finance,
            "Online Billing Portal",
            "Sovereign CI runner infrastructure renewal.",
        ));

        self.selected_event_id = self.events.first().map(|e| e.id);
    }

    /// Adds an event to the calendar and assigns an auto-incrementing ID.
    pub fn add_event(&mut self, mut event: CalendarEvent) -> u32 {
        let id = self.next_event_id;
        self.next_event_id += 1;
        event.id = id;
        self.events.push(event);
        id
    }

    /// Deletes an event by its unique ID.
    pub fn delete_event(&mut self, id: u32) -> bool {
        let prev_len = self.events.len();
        self.events.retain(|e| e.id != id);
        if self.selected_event_id == Some(id) {
            self.selected_event_id = self.events.first().map(|e| e.id);
        }
        self.events.len() < prev_len
    }

    /// Toggles completion state of a reminder or event.
    pub fn toggle_completed(&mut self, id: u32) -> bool {
        if let Some(ev) = self.events.iter_mut().find(|e| e.id == id) {
            ev.is_completed = !ev.is_completed;
            true
        } else {
            false
        }
    }

    /// Navigates to the previous month.
    pub fn prev_month(&mut self) {
        if self.view_month == 1 {
            self.view_month = 12;
            self.view_year = self.view_year.saturating_sub(1);
        } else {
            self.view_month -= 1;
        }
        self.selected_day = self.selected_day.min(days_in_month(self.view_year, self.view_month));
    }

    /// Navigates to the next month.
    pub fn next_month(&mut self) {
        if self.view_month == 12 {
            self.view_month = 1;
            self.view_year = self.view_year.saturating_add(1);
        } else {
            self.view_month += 1;
        }
        self.selected_day = self.selected_day.min(days_in_month(self.view_year, self.view_month));
    }

    /// Resets the browsed view to Today.
    pub fn go_to_today(&mut self) {
        self.view_year = self.now_year;
        self.view_month = self.now_month;
        self.selected_day = self.now_day;
    }

    /// Returns whether a category is enabled in the active filter.
    pub fn is_category_enabled(&self, cat: EventCategory) -> bool {
        let idx = cat as usize;
        if idx < self.enabled_categories.len() {
            self.enabled_categories[idx]
        } else {
            true
        }
    }

    /// Toggles category visibility in the active filter.
    pub fn toggle_category(&mut self, cat: EventCategory) {
        let idx = cat as usize;
        if idx < self.enabled_categories.len() {
            self.enabled_categories[idx] = !self.enabled_categories[idx];
        }
    }

    /// Returns all events matching the specified date.
    pub fn events_for_date(&self, year: u16, month: u8, day: u8) -> Vec<&CalendarEvent> {
        self.events
            .iter()
            .filter(|e| self.is_category_enabled(e.category) && e.occurs_on(year, month, day))
            .collect()
    }

    /// Returns events for currently selected day in view.
    pub fn events_for_selected_date(&self) -> Vec<&CalendarEvent> {
        self.events_for_date(self.view_year, self.view_month, self.selected_day)
    }

    /// Returns events for the entire currently browsed month.
    pub fn events_for_month(&self, year: u16, month: u8) -> Vec<&CalendarEvent> {
        let max_days = days_in_month(year, month);
        let mut list = Vec::new();
        for d in 1..=max_days {
            for ev in self.events_for_date(year, month, d) {
                if !list.iter().any(|item: &&CalendarEvent| item.id == ev.id) {
                    list.push(ev);
                }
            }
        }
        list
    }

    /// Exports all active events to standard iCalendar (.ics) RFC 5545 format.
    pub fn export_icalendar(&self) -> String {
        let mut ics = String::new();
        ics.push_str("BEGIN:VCALENDAR\r\n");
        ics.push_str("VERSION:2.0\r\n");
        ics.push_str("PRODID:-//NovaOS Enterprise//Kairos Calendar 1.0//EN\r\n");
        ics.push_str("CALSCALE:GREGORIAN\r\n");
        ics.push_str("METHOD:PUBLISH\r\n");

        for ev in &self.events {
            ics.push_str("BEGIN:VEVENT\r\n");
            ics.push_str(&format!("UID:kairos-{}@novaos.org\r\n", ev.id));
            ics.push_str(&format!("SUMMARY:{}\r\n", ev.title));
            ics.push_str(&format!("DESCRIPTION:{}\r\n", ev.description));
            ics.push_str(&format!("LOCATION:{}\r\n", ev.location));
            ics.push_str(&format!("DTSTART:{:04}{:02}{:02}T090000Z\r\n", ev.year, ev.month, ev.day));
            ics.push_str(&format!("DTEND:{:04}{:02}{:02}T100000Z\r\n", ev.year, ev.month, ev.day));
            ics.push_str(&format!("CATEGORIES:{}\r\n", ev.category.name()));
            ics.push_str("STATUS:CONFIRMED\r\n");
            ics.push_str("END:VEVENT\r\n");
        }

        ics.push_str("END:VCALENDAR\r\n");
        ics
    }
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leap_year_rules() {
        assert!(is_leap_year(2000), "Centurial divisible by 400 is leap");
        assert!(!is_leap_year(1900), "Centurial not divisible by 400 is not leap");
        assert!(is_leap_year(2024), "Divisible by 4 is leap");
        assert!(!is_leap_year(2026), "2026 is common year");
    }

    #[test]
    fn test_days_in_month_boundaries() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn test_day_of_week_accuracy() {
        // 2026-11-19 is Thursday (index 4)
        assert_eq!(day_of_week(2026, 11, 19), 4);
        // 2026-11-24 is Tuesday (index 2)
        assert_eq!(day_of_week(2026, 11, 24), 2);
        // 2026-01-01 is Thursday (index 4)
        assert_eq!(day_of_week(2026, 1, 1), 4);
    }

    #[test]
    fn test_event_crud_and_recurrence() {
        let mut state = CalendarState::new(2026, 11, 24);
        let id = state.add_event(CalendarEvent::new(
            0,
            "Daily Standup",
            2026,
            11,
            24,
            "09:30 AM",
            "10:00 AM",
            EventCategory::Work,
            "Online",
            "Sprint status",
        ));

        if let Some(ev) = state.events.iter_mut().find(|e| e.id == id) {
            ev.recurrence = RecurrenceRule::Daily;
        }

        // Daily event occurs today and tomorrow
        assert!(state.events_for_date(2026, 11, 24).iter().any(|e| e.id == id));
        assert!(state.events_for_date(2026, 11, 25).iter().any(|e| e.id == id));

        assert!(state.delete_event(id));
        assert!(!state.events_for_date(2026, 11, 24).iter().any(|e| e.id == id));
    }

    #[test]
    fn test_icalendar_export() {
        let state = CalendarState::new(2026, 11, 24);
        let ics = state.export_icalendar();
        assert!(ics.starts_with("BEGIN:VCALENDAR"));
        assert!(ics.contains("NovaOS 2.0 GA Worldwide Launch"));
        assert!(ics.ends_with("END:VCALENDAR\r\n"));
    }
}
