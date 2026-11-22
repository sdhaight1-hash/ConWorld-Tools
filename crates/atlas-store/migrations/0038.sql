-- Author-only, source-backed classification assignments; no inferred facts.
CREATE TABLE concept_classifications (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    subject_id BLOB NOT NULL,
    profile TEXT NOT NULL CHECK(profile IN ('culture','language','material','technology')),
    descriptor_id BLOB NOT NULL,
    descriptor_branch BLOB NOT NULL,
    descriptor_state_revision INTEGER NOT NULL,
    descriptor_revision INTEGER NOT NULL,
    source_id BLOB NOT NULL,
    starts_at INTEGER NOT NULL,
    ends_before INTEGER CHECK(ends_before IS NULL OR starts_at<ends_before),
    recorded_revision INTEGER NOT NULL,
    PRIMARY KEY(world_id,id),
    FOREIGN KEY(world_id,subject_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,source_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,descriptor_id,descriptor_branch,descriptor_state_revision,descriptor_revision)
        REFERENCES object_versions(world_id,object_id,branch_id,state_revision,revision),
    FOREIGN KEY(world_id,recorded_revision) REFERENCES author_revisions(world_id,revision)
) STRICT;
CREATE INDEX classification_subject ON concept_classifications(world_id,subject_id,starts_at,recorded_revision);
CREATE INDEX classification_profile ON concept_classifications(world_id,profile,starts_at,recorded_revision);

CREATE TABLE classification_withdrawals (
    world_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    fact_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    PRIMARY KEY(world_id,branch_id,fact_id),
    FOREIGN KEY(world_id,fact_id) REFERENCES concept_classifications(world_id,id),
    FOREIGN KEY(world_id,revision,branch_id) REFERENCES author_revisions(world_id,revision,branch_id)
) STRICT;
