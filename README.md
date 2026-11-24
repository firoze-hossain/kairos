# Kairos Calendar 📅

**Kairos** (Καιρός) is the sovereign, pure-Rust default Calendar & Event Scheduling application for **NovaOS**. In ancient classical philosophy, while *Chronos* measures quantitative, ticking clock time, *Kairos* represents the opportune, qualitative, and appointed moment for meetings, milestones, and personal commitments. Together, **Chronos (Clock)** and **Kairos (Calendar)** form a cohesive timekeeping and scheduling duo across the NovaOS ecosystem.

Kairos synthesizes the refined visual elegance of **macOS Calendar** (acrylic translucent headers, hero date sidebar badge, segmented view switcher, and detail inspector card) with the structured utilitarian clarity of **GNOME / Ubuntu Calendar** (compact month grid, vibrant category color pills, quick event add modals, and agenda view).

---

## 🌟 Key Features

### 1. 🗓️ Comprehensive Multi-View Engine
- **Month Grid**: Full monthly matrix with boundary day shading, multi-event badges (+N more), and current day highlight rings.
- **Week View**: 7-day multi-column time-block schedule with hourly guides (00:00 to 24:00) and current time horizontal rule.
- **Day View**: Focused single-day schedule with expanded event summary cards and visual conflict warnings.
- **Agenda View**: Linear, chronological feed of upcoming events grouped by date, ideal for daily standups and task reviews.
- **Year Overview**: 12-month miniature heat-map grid showing annual event density.

### 2. 🎨 Category Classification & Filtering
- **Six Standard Categories**: Work (Royal Blue), Personal (Emerald Green), Family (Amber Orange), Health (Crimson Red), Finance (Purple Amethyst), and System Tasks (Graphite Gray).
- **Interactive Sidebar Toggles**: Filter schedule dynamically by toggling categories on/off with immediate visual feedback.

### 3. 🔁 Recurrence & Event Management
- **Recurrence Rules**: Full support for Daily, Weekly, Monthly, and Annual recurrences with customizable intervals and end-date cutoffs.
- **Event Conflict Detection**: Identifies overlapping time blocks and provides visual warning indicators.
- **Event Metadata**: Title, start/end timestamps, location/room, category, description notes, and alarm notifications.

### 4. 🌐 RFC 5545 iCalendar Interoperability
- **Lossless `.ics` Export & Import**: Compatible with Google Calendar, Apple Calendar, Outlook, and Nextcloud.
- **Full VCALENDAR & VEVENT formatting**: Includes UID, DTSTAMP, DTSTART, DTEND, SUMMARY, DESCRIPTION, and CATEGORIES fields.

### 5. ⚡ Pure 2D Software Rasterizer
- Zero GPU or dynamic C library requirements.
- Double-buffered `CanvasSurface` with anti-aliased rounded rectangles, category pills, high-contrast text rendering, and acrylic status bars.
- 100% self-contained across desktop platforms and NovaOS bare-metal ring-3.

---

## 🏗️ Workspace Architecture

```
kairos/
├── Cargo.toml
├── crates/
│   ├── kairos-core/       # Domain logic: CalendarState, Event, Recurrence, Tomohiko Sakamoto math, RFC 5545
│   │   └── src/lib.rs
│   ├── kairos-ui/         # Pure 2D software renderer: Month, Week, Day, Agenda, Year views, sidebar, inspector
│   │   └── src/lib.rs
│   ├── kairos-platform/   # Cross-platform hardware RTC & time drivers (Linux, macOS, Windows, NovaOS)
│   │   └── src/lib.rs
│   └── kairos-app/        # Cross-platform CLI runner and interactive viewer
│       └── src/main.rs
└── novaos/                # NovaOS bare-metal ring-3 port
    ├── ffi.rs             # Ring-3 system call bindings (sys_write, sys_exit, sys_yield)
    ├── kairos.rs          # Standalone no_std / no_main ELF32 executable
    └── build.sh           # Automated build script creating KAIROS.ELF, CALENDAR.ELF, and PKG fixtures
```

---

## 🚀 Building & Running

### 1. Cross-Platform Desktop (Linux, macOS, Windows)

```bash
# Run self-test qualification suite
cargo run -p kairos-app -- --selftest

# Inspect scheduled events
cargo run -p kairos-app -- --events

# Render formatted terminal month grid
cargo run -p kairos-app -- --month

# Export standard RFC 5545 iCalendar stream
cargo run -p kairos-app -- --export-ics
```

### 2. Autonomous Unit & Integration Tests

```bash
cargo test
```
*Executes all tests across core mathematical algorithms, Tomohiko Sakamoto day-of-week anchors, leap year rules, recurrence expansions, iCalendar formatting, and 2D rasterization.*

### 3. NovaOS Bare-Metal Ring-3 Port

```bash
# Build i686-novaos ELF32 binaries and package fixtures
./novaos/build.sh
```
*Builds `kairos.elf`, creates `KAIROS.ELF` and `CALENDAR.ELF`, and generates `KAIROS.PKG` and `CALENDAR.PKG` metadata files inside `$NOVAOS_ROOT/tools/fixtures/`.*
