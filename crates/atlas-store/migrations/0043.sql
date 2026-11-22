-- Author-only links to existing sourced claims; no replicated fact or claim payload.
CREATE TABLE IF NOT EXISTS fact_annotations (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    family TEXT NOT NULL CHECK(family IN ('network','parentage','classification','site_area')),
    fact_id BLOB NOT NULL CHECK(length(fact_id)=16),
    claim_id BLOB NOT NULL,
    starts_at INTEGER NOT NULL,
    recorded_revision INTEGER NOT NULL,
    PRIMARY KEY(world_id,id),
    FOREIGN KEY(world_id,claim_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,recorded_revision) REFERENCES author_revisions(world_id,revision)
) STRICT;
CREATE INDEX IF NOT EXISTS fact_annotation_target
    ON fact_annotations(world_id,family,fact_id,recorded_revision);
CREATE INDEX IF NOT EXISTS fact_annotation_claim
    ON fact_annotations(world_id,claim_id,recorded_revision);

CREATE TABLE IF NOT EXISTS descriptor_retirements (
    world_id BLOB NOT NULL,
    descriptor_id BLOB NOT NULL,
    starts_at INTEGER NOT NULL,
    recorded_revision INTEGER NOT NULL,
    PRIMARY KEY(world_id,descriptor_id,recorded_revision),
    FOREIGN KEY(world_id,descriptor_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,recorded_revision) REFERENCES author_revisions(world_id,revision)
) STRICT;
