use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::BufRead;
use std::str::FromStr;

use chrono::naive::{NaiveDate, NaiveDateTime};
use chrono::{Duration, Timelike};
use regex::Regex;

#[derive(Clone)]
struct Shift {
    id: u32,
    date: NaiveDate,
    sleeping: [bool; 60],
}

struct Roster(HashMap<u32, Vec<Shift>>);
type Result<T> = ::std::result::Result<T, Box<Error>>;

impl Shift {
    fn new(id: u32, date: NaiveDate) -> Self {
        Self {
            id: id,
            date: date,
            sleeping: [false; 60],
        }
    }

    fn asleep(&self) -> u32 {
        self.sleeping.iter().map(|&s| if s { 1 } else { 0 }).sum()
    }

    fn sleep(&mut self, from: u32, to: u32) {
        for minute in from..to {
            self.sleeping[minute as usize] = true;
        }
    }
}

impl fmt::Debug for Shift {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Shift {{ id: {:?}, date: {:?}, asleep: {:?} }}",
            self.id,
            self.date,
            self.asleep()
        )?;
        Ok(())
    }
}

impl fmt::Display for Shift {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} Guard #{:<4}  ", self.date, self.id)?;
        for &sleep in self.sleeping.iter() {
            if sleep {
                write!(f, "#")?;
            } else {
                write!(f, ".")?;
            }
        }

        Ok(())
    }
}

impl Roster {
    fn new() -> Self {
        Roster(HashMap::new())
    }

    fn insert(&mut self, shift: Shift) {
        self.0.entry(shift.id).or_default().push(shift)
    }

    fn iter(&self) -> impl Iterator<Item = (&u32, &Vec<Shift>)> {
        self.0.iter()
    }
}

#[derive(Debug, Clone)]
enum GuardState {
    Missing,
    Awake,
    Asleep(u32),
}

fn shift_date(timestamp: NaiveDateTime) -> NaiveDate {
    if timestamp.hour() > 1 {
        timestamp.date() + Duration::days(1)
    } else {
        timestamp.date()
    }
}

#[derive(Debug, Clone)]
struct Door {
    state: GuardState,
    shift: Option<Shift>,
}

impl Door {
    fn new() -> Self {
        Self {
            state: GuardState::Missing,
            shift: None,
        }
    }

    fn last_shift(&mut self) -> Result<Option<Shift>> {
        Ok(self.shift.take())
    }

    fn process_state(&mut self, result: SleepState) -> Result<()> {
        let (state, sleep) = result;
        if let Some((start, stop)) = sleep {
            if let Some(ref mut shift) = self.shift {
                shift.sleep(start, stop);
            }
        }
        self.state = state;
        Ok(())
    }

    #[cfg_attr(feature = "cargo-clippy", allow(clippy::or_fun_call))]
    fn process_entry(&mut self, entry: &LogEntry) -> Result<Option<Shift>> {
        let date = shift_date(entry.timestamp);

        let minute = if entry.timestamp.date() == date {
            entry.timestamp.minute()
        } else {
            60
        };

        // If it is a new day, assume the guard left yesterday, and yield that state.
        let mut result = if self.shift.as_ref().map(|s| s.date != date).unwrap_or(false) {
            self.process_state(self.state.new_day()?)?;
            self.shift.take()
        } else {
            None
        };

        // Process the guard state at the door.
        {
            let (state, sleep) = match entry.entry {
                LogEntryValue::GuardBegins(id) => {
                    // If new guard has arrived, new shift.
                    if self.shift.as_ref().map(|s| s.id != id).unwrap_or(true) {
                        // Only yield the old shift if it was actually a shift, otherwise,
                        // assume result is already set correctly.
                        result = result.or(self.shift.replace(Shift::new(id, date)));
                    }

                    // Toggle the state machine to respect arrival
                    self.state.arrive(minute)
                }
                LogEntryValue::FallsAsleep => self.state.fall_asleep(entry.timestamp.minute()),
                LogEntryValue::WakesUp => self.state.wake_up(entry.timestamp.minute()),
            }?;

            if let Some((start, stop)) = sleep {
                if let Some(ref mut shift) = self.shift {
                    shift.sleep(start, stop);
                }
            }
            self.process_state((state, sleep))?;
        };

        Ok(result)
    }
}

type SleepState = (GuardState, Option<(u32, u32)>);

impl GuardState {
    fn new_day(&self) -> Result<SleepState> {
        match self {
            GuardState::Asleep(start) => Ok((GuardState::Missing, Some((*start, 60)))),
            GuardState::Awake => Ok((GuardState::Missing, None)),
            GuardState::Missing => Ok((GuardState::Missing, None)),
        }
    }

    fn fall_asleep(&self, now: u32) -> Result<SleepState> {
        match self {
            GuardState::Asleep(start) => Ok((GuardState::Asleep(*start), None)),
            GuardState::Awake => Ok((GuardState::Asleep(now), None)),
            GuardState::Missing => err!("No guard to fall asleep!"),
        }
    }

    fn wake_up(&self, now: u32) -> Result<SleepState> {
        match self {
            GuardState::Asleep(start) => Ok((GuardState::Awake, Some((*start, now)))),
            GuardState::Awake => Ok((GuardState::Awake, None)),
            GuardState::Missing => err!("No guard to wake up!"),
        }
    }

    fn arrive(&self, now: u32) -> Result<SleepState> {
        match self {
            GuardState::Asleep(start) => Ok((GuardState::Awake, Some((*start, now)))),
            GuardState::Awake => Ok((GuardState::Awake, None)),
            GuardState::Missing => Ok((GuardState::Awake, None)),
        }
    }
}

fn generate_shifts(entries: &[LogEntry]) -> Result<Roster> {
    let mut roster = Roster::new();
    let mut door = Door::new();

    for entry in entries {
        if let Some(shift) = door.process_entry(entry)? {
            roster.insert(shift);
        }
    }
    if let Some(shift) = door.last_shift()? {
        roster.insert(shift);
    }

    Ok(roster)
}

fn sleepiest_guard(shifts: &Roster) -> Result<(u32, Vec<Shift>)> {
    shifts
        .iter()
        .max_by_key(|(_, shifts)| shifts.iter().map(|s| s.asleep()).sum::<u32>())
        .ok_or_else(|| Box::<Error>::from("No guard was ever asleep".to_string()))
        .map(|(&gid, shifts)| (gid, shifts.clone()))
}

fn most_asleep_minute(shifts: &[Shift]) -> Result<u32> {
    most_asleep_minute_and_count(shifts).map(|(m, _)| m)
}

fn most_asleep_minute_and_count(shifts: &[Shift]) -> Result<(u32, u32)> {
    let mut asleep = [0u32; 60];
    for shift in shifts {
        for (m, _) in shift.sleeping.iter().enumerate().filter(|(_, &s)| s) {
            asleep[m] += 1;
        }
    }

    asleep
        .iter()
        .enumerate()
        .max_by_key(|(_, &c)| c)
        .ok_or_else(|| Box::<Error>::from("Guard was never asleep".to_string()))
        .map(|(m, c)| (m as u32, *c))
}

fn algorithm_part2(shifts: &Roster) -> Result<u32> {
    let mut mams = HashMap::new();

    for (&gid, gshifts) in shifts.iter() {
        mams.insert(gid, most_asleep_minute_and_count(gshifts)?);
    }

    mams.iter()
        .map(|(gid, (m, c))| (gid, m, c))
        .max_by_key(|(_, _, &c)| c)
        .ok_or_else(|| Box::<Error>::from("No guard was ever asleep".to_string()))
        .map(|(gid, m, _)| gid * m)
}

fn algorithm_part1(shifts: &Roster) -> Result<u32> {
    let (gid, g_shifts) = sleepiest_guard(shifts)?;
    let mam = most_asleep_minute(&g_shifts)?;
    Ok(mam * gid)
}

#[derive(Debug, PartialEq)]
enum LogEntryValue {
    GuardBegins(u32),
    FallsAsleep,
    WakesUp,
}

impl FromStr for LogEntryValue {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim() {
            "falls asleep" => return Ok(LogEntryValue::FallsAsleep),
            "wakes up" => return Ok(LogEntryValue::WakesUp),
            _ => {}
        };

        lazy_static! {
            static ref RE: Regex = Regex::new(r"Guard #(?P<id>[\d]+) begins shift").unwrap();
        };

        let cap = match RE.captures(s) {
            None => return err!("Can't parse log entry: {}", s),
            Some(cap) => cap,
        };

        Ok(LogEntryValue::GuardBegins(cap["id"].parse()?))
    }
}

#[derive(Debug, PartialEq)]
struct LogEntry {
    timestamp: NaiveDateTime,
    entry: LogEntryValue,
}

impl FromStr for LogEntry {
    type Err = Box<Error>;

    fn from_str(s: &str) -> Result<Self> {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                r"(?x)
                \[(?P<date>[\d:\-\s]+)\]
                \s+
                (?P<entry>.+)
                "
            )
            .unwrap();
        };

        let cap = match RE.captures(s) {
            None => return err!("Can't match entry: {}", s),
            Some(cap) => cap,
        };

        let timestamp = NaiveDateTime::parse_from_str(&cap["date"], "%Y-%m-%d %H:%M")?;
        let entry: LogEntryValue = cap["entry"].parse()?;

        Ok(Self {
            timestamp: timestamp,
            entry: entry,
        })
    }
}

fn parse_entries() -> Result<Vec<LogEntry>> {
    use crate::input;

    let mut entries: Vec<LogEntry> = input(4)?
        .lines()
        .map(|s| Ok(s?.parse()?))
        .collect::<Result<Vec<LogEntry>>>()?;

    entries.sort_by_key(|e| e.timestamp);

    Ok(entries)
}

pub(crate) fn main() -> Result<()> {
    let entries = parse_entries()?;
    let shifts = generate_shifts(&entries).unwrap();

    println!("Part 1: {}", algorithm_part1(&shifts)?);

    println!("Part 2: {}", algorithm_part2(&shifts)?);

    Ok(())
}

#[cfg(test)]
mod test {

    use chrono::naive::{NaiveDate, NaiveTime};

    use super::*;

    static LOG: &str = "[1518-11-01 00:00] Guard #10 begins shift
[1518-11-01 00:05] falls asleep
[1518-11-01 00:25] wakes up
[1518-11-01 00:30] falls asleep
[1518-11-01 00:55] wakes up
[1518-11-01 23:58] Guard #99 begins shift
[1518-11-02 00:40] falls asleep
[1518-11-02 00:50] wakes up
[1518-11-03 00:05] Guard #10 begins shift
[1518-11-03 00:24] falls asleep
[1518-11-03 00:29] wakes up
[1518-11-04 00:02] Guard #99 begins shift
[1518-11-04 00:36] falls asleep
[1518-11-04 00:46] wakes up
[1518-11-05 00:03] Guard #99 begins shift
[1518-11-05 00:45] falls asleep
[1518-11-05 00:55] wakes up";

    #[test]
    fn parse_entry() {
        let entry: LogEntry = LOG.split('\n').nth(0).unwrap().parse().unwrap();

        assert_eq!(
            entry,
            LogEntry {
                timestamp: NaiveDateTime::new(
                    NaiveDate::from_ymd(1518, 11, 1),
                    NaiveTime::from_hms(0, 0, 0)
                ),
                entry: LogEntryValue::GuardBegins(10)
            }
        )
    }

    #[test]
    fn make_shifts() {
        let entries: Vec<LogEntry> = LOG.split('\n').map(|s| s.parse().unwrap()).collect();

        let shifts = generate_shifts(&entries).unwrap();

        let g10_shifts = &shifts.0[&10];

        assert_eq!(
            format!("{}", g10_shifts[0]),
            "1518-11-01 Guard #10    .....####################.....#########################....."
                .to_string()
        );
        assert_eq!(
            format!("{}", g10_shifts[1]),
            "1518-11-03 Guard #10    ........................#####..............................."
                .to_string()
        );
        assert_eq!(g10_shifts.iter().map(|s| s.asleep()).sum::<u32>(), 50);
        assert_eq!(most_asleep_minute(g10_shifts).unwrap(), 24);
    }

    #[test]
    fn example_part1() {
        let entries: Vec<LogEntry> = LOG.split('\n').map(|s| s.parse().unwrap()).collect();
        let shifts = generate_shifts(&entries).unwrap();

        assert_eq!(algorithm_part1(&shifts).unwrap(), 240);
    }

    #[test]
    fn answer_part1() {
        let entries = parse_entries().unwrap();
        let shifts = generate_shifts(&entries).unwrap();

        assert_eq!(algorithm_part1(&shifts).unwrap(), 39584);
    }

    #[test]
    fn example_part2() {
        let entries: Vec<LogEntry> = LOG.split('\n').map(|s| s.parse().unwrap()).collect();
        let shifts = generate_shifts(&entries).unwrap();

        assert_eq!(algorithm_part2(&shifts).unwrap(), 4455);
    }

    #[test]
    fn answer_part2() {
        let entries = parse_entries().unwrap();
        let shifts = generate_shifts(&entries).unwrap();

        assert_eq!(algorithm_part2(&shifts).unwrap(), 55053);
    }
}
