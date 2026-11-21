use atlas_store::{
    query::{QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
    *,
};
use uuid::Uuid;

fn record(label: &str, start: i64, data: ObjectData) -> Record {
    Record {
        label: label.into(),
        valid: ValidInterval {
            start: Some(WorldDay(start)),
            end_exclusive: None,
        },
        data,
        source: None,
    }
}

fn command(world: &World, expected: AuthorRevision, changes: Vec<Change>) -> Command {
    Command {
        world: world.id,
        branch: world.main,
        expected,
        id: Uuid::new_v4(),
        author: "Role query fixture".into(),
        rationale: "Check historically recorded event roles".into(),
        scope: Scope::Author,
        changes,
    }
}

fn view(world: &World, revision: AuthorRevision, day: i64) -> ViewContext {
    ViewContext {
        world: world.id,
        branch: world.main,
        revision,
        day: WorldDay(day),
        scope: Scope::Author,
    }
}

fn query(participant: ObjectRef, role: Option<EventRole>, limit: usize) -> QuerySpec {
    QuerySpec {
        schema_version: QUERY_SCHEMA_VERSION,
        filter: QueryFilter::ParticipantEvents { participant, role },
        limit,
    }
}

#[test]
fn participant_event_search_respects_role_valid_time_and_author_corrections() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Valley", Scope::Author)?;
    let other = store.create_world("Elsewhere", Scope::Author)?;
    let alice = ObjectRef::new(world.id);
    let bob = ObjectRef::new(world.id);
    let meeting = ObjectRef::new(world.id);
    let hearing = ObjectRef::new(world.id);
    let first = store.apply(&command(
        &world,
        world.revision,
        vec![
            Change::Create(alice, record("Alice", 0, ObjectData::Person)),
            Change::Create(bob, record("Bob", 0, ObjectData::Person)),
            Change::Create(
                meeting,
                record(
                    "Guild meeting",
                    10,
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(10)),
                    },
                ),
            ),
            Change::Create(
                hearing,
                record(
                    "Court hearing",
                    20,
                    ObjectData::HistoricalEvent {
                        occurrence: EventDate::Exact(WorldDay(20)),
                    },
                ),
            ),
        ],
    ))?;
    let role_record = |label: &str, event, participant, role, day| {
        record(
            label,
            day,
            ObjectData::EventInvolvement {
                event,
                participant,
                role,
            },
        )
    };
    let involvement = ObjectRef::new(world.id);
    let later_involvement = ObjectRef::new(world.id);
    let witness = ObjectRef::new(world.id);
    let second = store.apply(&command(
        &world,
        first,
        vec![
            Change::Create(
                involvement,
                role_record("Guild attendance", meeting, alice, EventRole::Actor, 10),
            ),
            Change::Create(
                later_involvement,
                role_record("Court attendance", hearing, alice, EventRole::Actor, 20),
            ),
            Change::Create(
                witness,
                role_record("Witness at meeting", meeting, bob, EventRole::Witness, 10),
            ),
        ],
    ))?;
    let actor = query(alice, Some(EventRole::Actor), 100);
    let all = query(alice, None, 100);
    assert_eq!(
        store
            .execute_query(&actor, view(&world, second, 9))?
            .rows
            .len(),
        0
    );
    assert_eq!(
        store.execute_query(&actor, view(&world, second, 10))?.rows[0].object,
        meeting
    );
    assert_eq!(
        store
            .execute_query(&actor, view(&world, second, 25))?
            .rows
            .len(),
        2
    );
    assert_eq!(
        store
            .execute_query(&all, view(&world, second, 25))?
            .rows
            .len(),
        2
    );
    assert!(
        store
            .execute_query(&actor, view(&world, first, 25))?
            .rows
            .is_empty()
    );
    assert_eq!(
        store
            .execute_query(
                &query(bob, Some(EventRole::Witness), 100),
                view(&world, second, 25)
            )?
            .rows[0]
            .object,
        meeting
    );
    assert!(
        store
            .execute_query(
                &query(alice, Some(EventRole::Witness), 100),
                view(&world, second, 25)
            )?
            .rows
            .is_empty()
    );
    assert!(
        store
            .execute_query(&query(alice, None, 1), view(&world, second, 25))?
            .truncated
    );
    let fork = store.fork(&ForkCommand {
        world: world.id,
        parent: world.main,
        day: WorldDay(25),
        expected: second,
        id: Uuid::new_v4(),
        author: "Role query fixture".into(),
        rationale: "Preserve old event roles".into(),
        scope: Scope::Author,
    })?;
    let corrected = store.apply(&command(
        &world,
        fork.created_revision,
        vec![Change::Correct(
            involvement,
            role_record("Guild attendance", meeting, alice, EventRole::Witness, 10),
        )],
    ))?;
    assert_eq!(
        store
            .execute_query(&actor, view(&world, corrected, 25))?
            .rows
            .len(),
        1
    );
    assert_eq!(
        store
            .execute_query(
                &query(alice, Some(EventRole::Witness), 100),
                view(&world, corrected, 25)
            )?
            .rows[0]
            .object,
        meeting
    );
    assert_eq!(
        store
            .execute_query(&actor, view(&world, second, 25))?
            .rows
            .len(),
        2
    );
    let forked = ViewContext {
        branch: fork.id,
        ..view(&world, corrected, 25)
    };
    assert_eq!(store.execute_query(&actor, forked)?.rows.len(), 2);
    assert!(matches!(
        store.execute_query(
            &actor,
            ViewContext {
                scope: Scope::Public,
                ..view(&world, corrected, 25)
            }
        ),
        Err(Error::CapabilityDenied)
    ));
    assert!(matches!(
        store.execute_query(
            &query(ObjectRef::new(other.id), None, 100),
            view(&world, corrected, 25),
        ),
        Err(Error::WrongWorld)
    ));
    assert!(
        store
            .execute_query(&query(meeting, None, 100), view(&world, corrected, 25))?
            .rows
            .is_empty(),
        "a historical event identity must not be mistaken for a participant person"
    );
    Ok(())
}
