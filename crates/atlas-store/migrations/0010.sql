-- Explicit author-reviewed links to exact historical claim versions. This
-- stores a link decision only: no relationship, membership or inferred fact.
CREATE TABLE mention_reviews (
    world_id BLOB NOT NULL,
    source_id BLOB NOT NULL,
    source_branch_id BLOB NOT NULL,
    source_state_revision INTEGER NOT NULL,
    source_revision INTEGER NOT NULL,
    target_id BLOB NOT NULL,
    start_byte INTEGER NOT NULL CHECK(start_byte >= 0),
    end_byte INTEGER NOT NULL CHECK(end_byte > start_byte),
    decision TEXT NOT NULL CHECK(decision IN ('approved', 'dismissed')),
    revision INTEGER NOT NULL CHECK(revision > source_revision),
    source_kind TEXT GENERATED ALWAYS AS ('claim') VIRTUAL,
    target_kind TEXT GENERATED ALWAYS AS ('person') VIRTUAL,
    PRIMARY KEY (
        world_id, source_id, source_branch_id, source_state_revision,
        source_revision, start_byte, end_byte, target_id, revision
    ),
    FOREIGN KEY (world_id, source_id, source_kind) REFERENCES objects(world_id, id, kind),
    FOREIGN KEY (world_id, target_id, target_kind) REFERENCES objects(world_id, id, kind),
    FOREIGN KEY (world_id, source_id, source_branch_id, source_state_revision, source_revision)
        REFERENCES object_versions(world_id, object_id, branch_id, state_revision, revision),
    FOREIGN KEY (world_id, revision) REFERENCES author_revisions(world_id, revision)
) STRICT;
CREATE INDEX mention_reviews_target_revision ON mention_reviews(world_id, target_id, revision);
