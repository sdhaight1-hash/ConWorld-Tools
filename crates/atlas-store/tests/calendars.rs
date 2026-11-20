use atlas_store::*;
use tempfile::tempdir;
use uuid::Uuid;

fn years() -> CalendarRules {
    CalendarRules {
        name: "River calendar".into(),
        epoch: WorldDay(100),
        month_lengths: vec![10, 10],
        leap: Some(LeapCycle { every: 4, month: 2 }),
    }
}

#[test]
fn calendar_round_trips_across_the_epoch_and_leap_cycles() -> Result<(), Error> {
    let rules = years();
    for (year, month, day, world_day) in [
        (-4, 2, 11, 39),
        (-1, 2, 10, 99),
        (0, 1, 1, 100),
        (0, 2, 11, 120),
        (1, 1, 1, 121),
        (4, 2, 11, 201),
    ] {
        let date = CalendarDay { year, month, day };
        assert_eq!(
            rules.normalize(CalendarDate::Day(date), "")?,
            EventDate::Exact(WorldDay(world_day))
        );
        assert_eq!(rules.date_on(WorldDay(world_day))?, date);
    }
    for day in -1_000..=1_000 {
        let world_day = WorldDay(day);
        assert_eq!(
            rules.normalize(CalendarDate::Day(rules.date_on(world_day)?), "")?,
            EventDate::Exact(world_day)
        );
    }
    assert!(matches!(
        rules.normalize(
            CalendarDate::Day(CalendarDay {
                year: -1,
                month: 2,
                day: 11
            }),
            ""
        ),
        Err(Error::InvalidCalendarDate)
    ));
    assert!(matches!(
        rules.normalize(CalendarDate::Month { year: 0, month: 0 }, "invalid"),
        Err(Error::InvalidCalendarDate)
    ));
    assert!(matches!(
        rules.normalize(CalendarDate::Year(i64::MAX), "future"),
        Err(Error::CalendarOutOfRange)
    ));
    Ok(())
}

#[test]
fn partial_calendar_dates_preserve_inclusive_bounds_and_labels() -> Result<(), Error> {
    let rules = years();
    for (date, label, first, last) in [
        (CalendarDate::Year(-1), "Year before the epoch", 80, 99),
        (
            CalendarDate::Month { year: 0, month: 2 },
            "Highwater of year zero",
            110,
            120,
        ),
    ] {
        let result = rules.normalize(date, label)?;
        assert_eq!(
            result,
            EventDate::Uncertain {
                earliest: Some(WorldDay(first)),
                latest: Some(WorldDay(last)),
                label: label.into()
            }
        );
        assert_eq!(result.certainty_on(WorldDay(last)), Certainty::Possibly);
        assert_eq!(
            result.certainty_on(WorldDay(last + 1)),
            Certainty::NotEstablished
        );
    }
    assert!(matches!(
        rules.normalize(CalendarDate::Year(0), " "),
        Err(Error::InvalidInput(_))
    ));
    Ok(())
}

#[test]
fn stored_calendars_are_world_scoped_and_updates_keep_prior_rules() -> Result<(), Error> {
    let dir = tempdir()?;
    let file = dir.path().join("archive.sqlite");
    let mut store = Store::open(&file)?;
    let world = store.create_world("River", Scope::Author)?;
    let other = store.create_world("Coast", Scope::Author)?;
    let id = CalendarId(Uuid::new_v4());
    let mut cmd = CalendarCommand {
        world: world.id,
        calendar: id,
        expected: world.revision,
        command_id: Uuid::new_v4(),
        author: "Archivist".into(),
        rationale: "Define the date system".into(),
        scope: Scope::Author,
        rules: years(),
    };
    let first = store.register_calendar(&cmd)?;
    assert!(
        matches!(store.register_calendar(&cmd), Err(Error::DuplicateCommand(rev)) if rev == first)
    );
    cmd.command_id = Uuid::new_v4();
    assert!(
        matches!(store.register_calendar(&cmd), Err(Error::RevisionConflict { actual, .. }) if actual == first)
    );
    cmd.expected = first;
    cmd.rules.month_lengths[0] = 0;
    assert!(matches!(
        store.register_calendar(&cmd),
        Err(Error::InvalidCalendar)
    ));
    assert_eq!(store.world(world.id, Scope::Author)?.revision, first);
    cmd.rules.month_lengths[0] = 11;
    let historic_date = store
        .calendar(world.id, id, first, Scope::Author)?
        .rules
        .normalize(
            CalendarDate::Month { year: 0, month: 2 },
            "Highwater, year zero",
        )?;
    let succession = ObjectRef::new(world.id);
    let recorded = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: first,
        id: Uuid::new_v4(),
        author: "Archivist".into(),
        rationale: "Record the succession".into(),
        scope: Scope::Author,
        changes: vec![Change::Create(
            succession,
            Record {
                label: "River succession".into(),
                valid: ValidInterval::default(),
                data: ObjectData::HistoricalEvent {
                    occurrence: historic_date.clone(),
                },
                source: None,
            },
        )],
    })?;
    cmd.expected = recorded;
    let second = store.register_calendar(&cmd)?;
    drop(store);

    let store = Store::open(file)?;
    let original = store.calendar(world.id, id, first, Scope::Author)?;
    let revised = store.calendar(world.id, id, second, Scope::Author)?;
    let date = CalendarDate::Day(CalendarDay {
        year: 0,
        month: 2,
        day: 1,
    });
    assert_eq!(original.registered_at, first);
    assert_eq!(
        original.rules.normalize(date, "")?,
        EventDate::Exact(WorldDay(110))
    );
    assert_eq!(
        revised.rules.normalize(date, "")?,
        EventDate::Exact(WorldDay(111))
    );
    assert!(matches!(
        store.calendar(world.id, id, world.revision, Scope::Author),
        Err(Error::NotFound)
    ));
    assert!(matches!(
        store.calendar(other.id, id, other.revision, Scope::Author),
        Err(Error::NotFound)
    ));
    assert!(matches!(
        store.calendar(world.id, id, second, Scope::Public),
        Err(Error::CapabilityDenied)
    ));
    let event = store
        .get_object(
            succession,
            ViewContext {
                world: world.id,
                branch: world.main,
                revision: second,
                day: WorldDay(110),
                scope: Scope::Author,
            },
        )?
        .unwrap();
    assert_eq!(
        event.record.data,
        ObjectData::HistoricalEvent {
            occurrence: historic_date
        }
    );
    store.verify_foreign_keys(Scope::Author)
}
