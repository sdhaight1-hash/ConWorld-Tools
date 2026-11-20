use crate::{AuthorRevision, Error, EventDate, Scope, WorldDay, WorldId};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CalendarId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LeapCycle {
    pub every: u16,
    pub month: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarRules {
    pub name: String,
    /// World day of year zero, month one, day one.
    pub epoch: WorldDay,
    /// Month lengths in calendar order, before leap adjustment.
    pub month_lengths: Vec<u16>,
    pub leap: Option<LeapCycle>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CalendarDay {
    pub year: i64,
    pub month: u16,
    pub day: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CalendarDate {
    Year(i64),
    Month { year: i64, month: u16 },
    Day(CalendarDay),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CalendarVersion {
    pub id: CalendarId,
    pub registered_at: AuthorRevision,
    pub rules: CalendarRules,
}

#[derive(Clone, Debug)]
pub struct CalendarCommand {
    pub world: WorldId,
    pub calendar: CalendarId,
    pub expected: AuthorRevision,
    pub command_id: Uuid,
    pub author: String,
    pub rationale: String,
    pub scope: Scope,
    pub rules: CalendarRules,
}

impl CalendarRules {
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.name.trim().is_empty()
            || self.month_lengths.is_empty()
            || self.month_lengths.len() > usize::from(u16::MAX)
            || self.month_lengths.contains(&0)
        {
            return Err(Error::InvalidCalendar);
        }
        if let Some(leap) = self.leap
            && (leap.every == 0
                || leap.month == 0
                || self
                    .month_lengths
                    .get(usize::from(leap.month - 1))
                    .is_none_or(|length| *length == u16::MAX))
        {
            return Err(Error::InvalidCalendar);
        }
        Ok(())
    }

    /// Converts exact dates or inclusive bounds into the world's normalized day axis.
    pub fn normalize(&self, date: CalendarDate, label: &str) -> Result<EventDate, Error> {
        self.validate()?;
        let (year, month, day) = match date {
            CalendarDate::Year(year) => (year, None, None),
            CalendarDate::Month { year, month } => (year, Some(month), None),
            CalendarDate::Day(date) => (date.year, Some(date.month), Some(date.day)),
        };
        let mut start = i128::from(self.epoch.0) + self.days_before_year(year);
        let length = if let Some(month) = month {
            if !(1..=self.month_lengths.len()).contains(&usize::from(month)) {
                return Err(Error::InvalidCalendarDate);
            }
            start += (1..month)
                .map(|m| i128::from(self.month_length(year, m)))
                .sum::<i128>();
            i128::from(self.month_length(year, month))
        } else {
            self.base_year_days() + i128::from(self.is_leap(year))
        };
        if let Some(day) = day {
            if day == 0 || i128::from(day) > length {
                return Err(Error::InvalidCalendarDate);
            }
            return Ok(EventDate::Exact(checked_day(start + i128::from(day) - 1)?));
        }
        if label.trim().is_empty() {
            return Err(Error::InvalidInput("partial calendar date label"));
        }
        Ok(EventDate::Uncertain {
            earliest: Some(checked_day(start)?),
            latest: Some(checked_day(start + length - 1)?),
            label: label.into(),
        })
    }

    pub fn date_on(&self, day: WorldDay) -> Result<CalendarDay, Error> {
        self.validate()?;
        let base = self.base_year_days();
        let (cycle, leap_days) = self.leap.map_or((1, 0), |leap| (i128::from(leap.every), 1));
        let cycle_days = cycle * base + leap_days;
        let elapsed = i128::from(day.0) - i128::from(self.epoch.0);
        let year_start = elapsed.div_euclid(cycle_days) * cycle;
        let mut offset = elapsed.rem_euclid(cycle_days);
        let mut year = year_start;
        if offset >= base + leap_days {
            offset -= base + leap_days;
            year += 1 + offset / base;
            offset %= base;
        }
        let year = i64::try_from(year).map_err(|_| Error::CalendarOutOfRange)?;
        for month in 1..=self.month_lengths.len() as u16 {
            let length = i128::from(self.month_length(year, month));
            if offset < length {
                return Ok(CalendarDay {
                    year,
                    month,
                    day: (offset + 1) as u16,
                });
            }
            offset -= length;
        }
        Err(Error::InvalidCalendar)
    }

    fn base_year_days(&self) -> i128 {
        self.month_lengths.iter().copied().map(i128::from).sum()
    }

    fn is_leap(&self, year: i64) -> bool {
        self.leap
            .is_some_and(|leap| year.rem_euclid(i64::from(leap.every)) == 0)
    }

    fn month_length(&self, year: i64, month: u16) -> u16 {
        let extra = self.is_leap(year) && self.leap.is_some_and(|leap| leap.month == month);
        self.month_lengths[usize::from(month - 1)] + u16::from(extra)
    }

    fn days_before_year(&self, year: i64) -> i128 {
        let year = i128::from(year);
        let extra = self.leap.map_or(0, |leap| {
            // Each cycle begins with a leap year, including the cycle at year zero.
            let every = i128::from(leap.every);
            year.div_euclid(every) + i128::from(year.rem_euclid(every) != 0)
        });
        year * self.base_year_days() + extra
    }
}

fn checked_day(day: i128) -> Result<WorldDay, Error> {
    i64::try_from(day)
        .map(WorldDay)
        .map_err(|_| Error::CalendarOutOfRange)
}
