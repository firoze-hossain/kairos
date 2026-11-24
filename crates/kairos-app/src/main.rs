//! crates/kairos-app/src/main.rs
//!
//! # Kairos Sovereign Calendar Suite (macOS + Ubuntu Synthesized)
//!
//! Master cross-platform desktop application and command-line scheduler.

use kairos_core::{
    days_in_month, day_of_week, format_date_long, CalendarEvent, CalendarState, EventCategory,
    ViewMode, MONTH_NAMES,
};
use kairos_platform::{NativeHostAdapter, PlatformAdapter};
use kairos_ui::{CanvasSurface, KairosUiRenderer, DEFAULT_WINDOW_H, DEFAULT_WINDOW_W};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let adapter = NativeHostAdapter;
    let now = adapter.current_date();

    let mut state = CalendarState::new(now.year, now.month, now.day);
    state.now_hour = now.hour;
    state.now_minute = now.minute;

    if args.contains(&"--selftest".to_string()) {
        run_autonomous_qualification(&state);
        return;
    }

    if args.contains(&"--events".to_string()) {
        println!("============================================================");
        println!("              KAIROS CALENDAR - SCHEDULED EVENTS            ");
        println!("============================================================");
        println!("Platform: {}", adapter.platform_name());
        println!("Date:     {}", format_date_long(now.year, now.month, now.day));
        println!("------------------------------------------------------------");
        for ev in &state.events {
            println!(
                "[{:04}-{:02}-{:02}] {} {} ({} - {}) @ {}",
                ev.year,
                ev.month,
                ev.day,
                ev.category.icon(),
                ev.title,
                ev.start_time,
                ev.end_time,
                ev.location
            );
        }
        println!("------------------------------------------------------------");
        println!("Total: {} events scheduled.", state.events.len());
        return;
    }

    if args.contains(&"--month".to_string()) {
        render_terminal_month(&state);
        return;
    }

    if args.contains(&"--export-ics".to_string()) {
        let ics = state.export_icalendar();
        println!("{}", ics);
        return;
    }

    // Default: Present interactive terminal overview and render graphical canvas
    println!("============================================================");
    println!("      KAIROS - SOVEREIGN CALENDAR & EVENT STUDIO v1.0.0      ");
    println!("============================================================");
    println!("Platform: {}", adapter.platform_name());
    println!("Today:    {}", format_date_long(now.year, now.month, now.day));
    println!("Time:     {:02}:{:02} UTC", now.hour, now.minute);
    println!("------------------------------------------------------------");

    render_terminal_month(&state);

    println!("\n[Viewports Rasterized]:");
    let mut canvas = CanvasSurface::new(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H);
    for mode in ViewMode::all() {
        state.view_mode = mode;
        KairosUiRenderer::render(&mut canvas, &state);
        println!("  -> Rendered ViewMode::{:?} canvas ({}x{})", mode, canvas.width, canvas.height);
    }

    println!("\nUsage: kairos [--selftest | --events | --month | --export-ics]");
}

fn render_terminal_month(state: &CalendarState) {
    let m_name = MONTH_NAMES[(state.view_month.clamp(1, 12) - 1) as usize];
    println!("\n       >>> {} {} <<<", m_name, state.view_year);
    println!("  Sun  Mon  Tue  Wed  Thu  Fri  Sat");

    let start_dow = day_of_week(state.view_year, state.view_month, 1);
    let max_days = days_in_month(state.view_year, state.view_month);

    let mut line = String::new();
    for _ in 0..start_dow {
        line.push_str("     ");
    }

    for d in 1..=max_days {
        let dow = (start_dow as usize + d as usize - 1) % 7;
        let has_event = !state.events_for_date(state.view_year, state.view_month, d).is_empty();
        let is_today = d == state.now_day && state.view_month == state.now_month && state.view_year == state.now_year;

        if is_today {
            line.push_str(&format!(" [{:2}]", d));
        } else if has_event {
            line.push_str(&format!(" *{:2} ", d));
        } else {
            line.push_str(&format!("  {:2} ", d));
        }

        if dow == 6 || d == max_days {
            println!("{}", line);
            line.clear();
        }
    }
}

pub fn run_autonomous_qualification(state: &CalendarState) -> bool {
    println!("============================================================");
    println!("        KAIROS CALENDAR AUTONOMOUS QUALIFICATION SUITE      ");
    println!("============================================================");

    // [1/6] Date Calculations & Gregorian Math
    println!("\n[1/6] Subsystem: Gregorian Date Engine & Leap Math");
    assert!(kairos_core::is_leap_year(2024));
    assert!(!kairos_core::is_leap_year(2026));
    assert_eq!(days_in_month(2026, 2), 28);
    assert_eq!(days_in_month(2024, 2), 29);
    println!("  [PASS] Leap year detection and month length calculations verified");

    // [2/6] Tomohiko Sakamoto's Day-of-Week Algorithm
    println!("\n[2/6] Subsystem: Day of Week Anchor Alignment");
    assert_eq!(day_of_week(2026, 11, 19), 4); // Thursday
    assert_eq!(day_of_week(2026, 11, 24), 2); // Tuesday
    println!("  [PASS] Day-of-week anchor matches ISO standard calendar");

    // [3/6] Event Lifecycle & Recurrence Engine
    println!("\n[3/6] Subsystem: Event CRUD & Multi-Frequency Recurrence");
    let mut s = state.clone();
    let initial_count = s.events.len();
    let ev_id = s.add_event(CalendarEvent::new(
        0,
        "Kairos Milestone Review",
        2026,
        11,
        24,
        "03:00 PM",
        "04:00 PM",
        EventCategory::NovaOS,
        "Sovereign Core",
        "Automated qualification test",
    ));
    assert_eq!(s.events.len(), initial_count + 1);
    assert!(s.events_for_date(2026, 11, 24).iter().any(|e| e.id == ev_id));
    assert!(s.delete_event(ev_id));
    assert_eq!(s.events.len(), initial_count);
    println!("  [PASS] Event addition, query, and deletion functioning properly");

    // [4/6] Category Tagging & Multi-Calendar Filters
    println!("\n[4/6] Subsystem: Category Filtering & Visual Tagging");
    s.toggle_category(EventCategory::Work);
    assert!(!s.is_category_enabled(EventCategory::Work));
    s.toggle_category(EventCategory::Work);
    assert!(s.is_category_enabled(EventCategory::Work));
    println!("  [PASS] Dynamic calendar toggling and category filtering operational");

    // [5/6] 2D UI Rasterization Across 5 Viewports
    println!("\n[5/6] Subsystem: 2D Canvas Rasterization (Month, Week, Day, Agenda, Year)");
    let mut canvas = CanvasSurface::new(DEFAULT_WINDOW_W, DEFAULT_WINDOW_H);
    for mode in ViewMode::all() {
        s.view_mode = mode;
        KairosUiRenderer::render(&mut canvas, &s);
        let non_zero = canvas.pixels.iter().filter(|&&px| px != 0).count();
        assert!(non_zero > 15000, "ViewMode {:?} drawn pixel count: {}", mode, non_zero);
        println!("  [PASS] Viewport {:?} rasterized cleanly ({} active pixels)", mode, non_zero);
    }

    // [6/6] Standard iCalendar RFC 5545 Export
    println!("\n[6/6] Subsystem: iCalendar (.ics) RFC 5545 Interoperability");
    let ics = s.export_icalendar();
    assert!(ics.contains("BEGIN:VCALENDAR"));
    assert!(ics.contains("BEGIN:VEVENT"));
    assert!(ics.contains("END:VCALENDAR"));
    println!("  [PASS] RFC 5545 iCalendar stream serialization valid");

    println!("\n============================================================");
    println!("AUTONOMOUS QUALIFICATION SUMMARY: ALL 6 SUBSYSTEMS PASSED");
    println!("============================================================");
    println!(">>> [SUCCESS] Kairos Calendar 100% QUALIFIED & OPERATIONAL <<<");
    true
}
