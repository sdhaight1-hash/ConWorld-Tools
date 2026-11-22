-- Extend the canonical registry, preserving every previously assigned object ID.
CREATE TABLE objects_v30 (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id)=16),
    kind TEXT NOT NULL CHECK(kind IN ('person','historical_event','source','claim',
        'relationship','event_involvement','actor_action','definition','organization','household','family_house')),
    PRIMARY KEY(world_id,id),
    UNIQUE(world_id,id,kind)
) STRICT;
INSERT INTO objects_v30 SELECT world_id,id,kind FROM objects;
DROP TABLE objects;
ALTER TABLE objects_v30 RENAME TO objects;

-- Main-branch author metadata, never reader-visible by default.
CREATE TABLE IF NOT EXISTS network_facts (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    subject_id BLOB NOT NULL,
    target_id BLOB NOT NULL CHECK(subject_id<>target_id),
    source_id BLOB NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('membership','marriage','succession','confirmed_cause','alleged_cause')),
    role TEXT NOT NULL CHECK(length(trim(role))>0),
    starts_at INTEGER NOT NULL,
    ends_before INTEGER CHECK(ends_before IS NULL OR starts_at<ends_before),
    recorded_revision INTEGER NOT NULL,
    withdrawn_revision INTEGER CHECK(withdrawn_revision IS NULL OR withdrawn_revision>recorded_revision),
    PRIMARY KEY(world_id,id),
    FOREIGN KEY(world_id,subject_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,target_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,source_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,recorded_revision) REFERENCES author_revisions(world_id,revision),
    FOREIGN KEY(world_id,withdrawn_revision) REFERENCES author_revisions(world_id,revision)
) STRICT;
CREATE INDEX IF NOT EXISTS network_subject ON network_facts(world_id,subject_id,kind,recorded_revision);
CREATE INDEX IF NOT EXISTS network_target ON network_facts(world_id,target_id,kind,recorded_revision);
CREATE TABLE IF NOT EXISTS ghost_proposals (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    claim_id BLOB NOT NULL,
    claim_branch BLOB NOT NULL,
    claim_state_revision INTEGER NOT NULL,
    claim_revision INTEGER NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL CHECK(end_byte>start_byte),
    label TEXT NOT NULL CHECK(length(trim(label))>0),
    kind TEXT NOT NULL CHECK(kind IN ('person','organization','household','family_house')),
    day INTEGER NOT NULL,
    recorded_revision INTEGER NOT NULL,
    reviewed_revision INTEGER,
    promoted_id BLOB,
    PRIMARY KEY(world_id,id),
    FOREIGN KEY(world_id,claim_id,claim_branch,claim_state_revision,claim_revision)
        REFERENCES object_versions(world_id,object_id,branch_id,state_revision,revision),
    FOREIGN KEY(world_id,recorded_revision) REFERENCES author_revisions(world_id,revision),
    FOREIGN KEY(world_id,reviewed_revision) REFERENCES author_revisions(world_id,revision),
    FOREIGN KEY(world_id,promoted_id) REFERENCES objects(world_id,id),
    CHECK(reviewed_revision IS NULL OR reviewed_revision>recorded_revision),
    CHECK(promoted_id IS NULL OR reviewed_revision IS NOT NULL)
) STRICT;
CREATE INDEX IF NOT EXISTS ghost_by_claim ON ghost_proposals(world_id,claim_id,recorded_revision);
