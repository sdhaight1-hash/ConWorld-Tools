-- Dedicated site area values; not a generic property store or spatial geometry.
CREATE TABLE IF NOT EXISTS site_area_facts (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    place_id BLOB NOT NULL,
    descriptor_id BLOB NOT NULL,
    descriptor_branch BLOB NOT NULL,
    descriptor_state_revision INTEGER NOT NULL,
    descriptor_revision INTEGER NOT NULL,
    source_id BLOB NOT NULL,
    amount INTEGER,
    unit TEXT,
    starts_at INTEGER NOT NULL,
    ends_before INTEGER CHECK(ends_before IS NULL OR starts_at<ends_before),
    recorded_revision INTEGER NOT NULL,
    corrects BLOB,
    PRIMARY KEY(world_id,id),
    FOREIGN KEY(world_id,place_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,source_id) REFERENCES objects(world_id,id),
    FOREIGN KEY(world_id,descriptor_id,descriptor_branch,descriptor_state_revision,descriptor_revision)
        REFERENCES object_versions(world_id,object_id,branch_id,state_revision,revision),
    FOREIGN KEY(world_id,corrects) REFERENCES site_area_facts(world_id,id),
    FOREIGN KEY(world_id,recorded_revision) REFERENCES author_revisions(world_id,revision),
    CHECK((amount IS NULL AND unit IS NULL) OR
          (amount IS NOT NULL AND amount>=0 AND unit IS NOT NULL AND unit IN ('ha','m2'))),
    CHECK(corrects IS NULL OR corrects<>id)
) STRICT;
CREATE INDEX IF NOT EXISTS site_area_place ON site_area_facts(world_id,place_id,starts_at,recorded_revision);
CREATE INDEX IF NOT EXISTS site_area_source ON site_area_facts(world_id,source_id,recorded_revision);
CREATE INDEX IF NOT EXISTS site_area_descriptor ON site_area_facts(world_id,descriptor_id,recorded_revision);
CREATE TABLE IF NOT EXISTS site_area_withdrawals (
    world_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    fact_id BLOB NOT NULL,
    revision INTEGER NOT NULL,
    PRIMARY KEY(world_id,branch_id,fact_id),
    FOREIGN KEY(world_id,fact_id) REFERENCES site_area_facts(world_id,id),
    FOREIGN KEY(world_id,revision,branch_id) REFERENCES author_revisions(world_id,revision,branch_id)
) STRICT;
