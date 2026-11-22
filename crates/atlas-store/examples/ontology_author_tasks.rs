//! AE-01 baseline: what a schema-36 author can preserve as attributed testimony.
use atlas_store::{
    query::{ObjectKindFilter, QUERY_SCHEMA_VERSION, QueryFilter, QuerySpec},
    *,
};
use uuid::Uuid;

fn main() -> Result<(), Error> {
    let mut store = Store::open(":memory:")?;
    let world = store.create_world("Estuary", Scope::Author)?;
    let source = ObjectRef::new(world.id);
    let event = ObjectRef::new(world.id);
    let create = |object, label: &str, data, source, valid| {
        Change::Create(
            object,
            Record {
                label: label.into(),
                data,
                source,
                valid,
            },
        )
    };
    let mut changes = vec![
        create(
            source,
            "Field notebook",
            ObjectData::Source {
                citation: "Survey folio 3".into(),
            },
            None,
            ValidInterval::default(),
        ),
        create(
            event,
            "Estuary survey",
            ObjectData::HistoricalEvent {
                occurrence: EventDate::Exact(WorldDay(10)),
            },
            None,
            ValidInterval::default(),
        ),
    ];
    let tasks = [
        (
            "language",
            "The survey recorded Estuary speech in port records.",
            "Estuary speech",
        ),
        (
            "area",
            "The bronze working site measured twelve hectares.",
            "twelve hectares",
        ),
        (
            "medium",
            "The aetheric drive used amber as its medium.",
            "amber",
        ),
    ];
    let mut expected = Vec::new();
    for (predicate, statement, phrase) in tasks {
        let claim = ObjectRef::new(world.id);
        changes.push(create(
            claim,
            predicate,
            ObjectData::Claim {
                subject: event,
                predicate: predicate.into(),
                statement: statement.into(),
                audience: ClaimAudience::Author,
            },
            Some(source),
            ValidInterval {
                start: Some(WorldDay(10)),
                end_exclusive: None,
            },
        ));
        expected.push((claim, phrase));
    }
    let revision = store.apply(&Command {
        world: world.id,
        branch: world.main,
        expected: world.revision,
        id: Uuid::new_v4(),
        author: "Field researcher".into(),
        rationale: "Record survey testimony".into(),
        scope: Scope::Author,
        changes,
    })?;
    for (claim, phrase) in expected {
        let query = QuerySpec {
            schema_version: QUERY_SCHEMA_VERSION,
            limit: 10,
            filter: QueryFilter::And(vec![
                QueryFilter::Kind(ObjectKindFilter::Claim),
                QueryFilter::HasSource(source),
                QueryFilter::ClaimFullTextPhrase(phrase.into()),
            ]),
        };
        for day in [9, 10] {
            let view = ViewContext {
                world: world.id,
                branch: world.main,
                day: WorldDay(day),
                revision,
                scope: Scope::Author,
            };
            let result = store.execute_query(&query, view)?;
            let actual: Vec<_> = result.rows.iter().map(|row| row.object).collect();
            assert_eq!(actual, if day == 10 { vec![claim] } else { vec![] });
        }
        println!("{phrase}: original sourced claim found on day 10 only");
    }
    Ok(())
}
