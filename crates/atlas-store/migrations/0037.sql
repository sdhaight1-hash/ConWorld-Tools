-- Extend canonical identity registry with Concept; preserve existing kinds, and revision history.
CREATE TABLE objects_v37 (
    world_id BLOB NOT NULL REFERENCES worlds(id),
    id BLOB NOT NULL CHECK(length(id)=16),
    kind TEXT NOT NULL CHECK(kind IN ('person','historical_event','source','claim',
        'relationship','event_involvement','actor_action','definition','organization','household','family_house','faction','conflict','office','title','asset','population','conflict_issue','conflict_phase','succession_rule','concept')),
    PRIMARY KEY(world_id,id),
    UNIQUE(world_id,id,kind)
) STRICT;
INSERT INTO objects_v37 SELECT world_id,id,kind FROM objects;
DROP TABLE objects;
ALTER TABLE objects_v37 RENAME TO objects;

CREATE TABLE ghost_proposals_v37 (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    claim_id BLOB NOT NULL,
    claim_branch BLOB NOT NULL,
    claim_state_revision INTEGER NOT NULL,
    claim_revision INTEGER NOT NULL,
    start_byte INTEGER NOT NULL,
    end_byte INTEGER NOT NULL CHECK(end_byte>start_byte),
    label TEXT NOT NULL CHECK(length(trim(label))>0),
    kind TEXT NOT NULL CHECK(kind IN ('person','organization','household','family_house','faction','conflict','office','title','asset','population','conflict_issue','conflict_phase','succession_rule','concept')),
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
INSERT INTO ghost_proposals_v37 SELECT * FROM ghost_proposals;
DROP TABLE ghost_proposals;
ALTER TABLE ghost_proposals_v37 RENAME TO ghost_proposals;
CREATE INDEX ghost_by_claim ON ghost_proposals(world_id,claim_id,recorded_revision);

