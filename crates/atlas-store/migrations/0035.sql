DROP TRIGGER metadata_withdrawal_validate;
CREATE TABLE network_facts_v35 (
    world_id BLOB NOT NULL,
    id BLOB NOT NULL CHECK(length(id)=16),
    subject_id BLOB NOT NULL,
    target_id BLOB NOT NULL CHECK(subject_id<>target_id),
    source_id BLOB NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('membership','marriage','succession','confirmed_cause','alleged_cause','conflict_position','office_holding','office_of','title_holding','title_of','ownership','issue_of','phase_of','rule_of','population_member','heir_designation','identity_redirect','alliance','settlement')),
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
INSERT INTO network_facts_v35 SELECT * FROM network_facts;
DROP TABLE network_facts;
ALTER TABLE network_facts_v35 RENAME TO network_facts;
CREATE INDEX network_subject ON network_facts(world_id,subject_id,kind,recorded_revision);
CREATE INDEX network_target ON network_facts(world_id,target_id,kind,recorded_revision);
CREATE TRIGGER metadata_withdrawal_validate BEFORE INSERT ON metadata_withdrawals
WHEN (new.kind='network' AND NOT EXISTS(SELECT 1 FROM network_facts WHERE world_id=new.world_id AND id=new.fact_id))
  OR (new.kind='parentage' AND NOT EXISTS(SELECT 1 FROM accepted_parentage WHERE world_id=new.world_id AND id=new.fact_id))
BEGIN SELECT RAISE(ABORT,'unknown metadata fact'); END;
