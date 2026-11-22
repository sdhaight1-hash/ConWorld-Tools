-- Branch-local corrections, recorded in author history without modifying inherited facts.
CREATE TABLE IF NOT EXISTS metadata_withdrawals (
    world_id BLOB NOT NULL,
    branch_id BLOB NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('network','parentage')),
    fact_id BLOB NOT NULL CHECK(length(fact_id)=16),
    revision INTEGER NOT NULL,
    PRIMARY KEY(world_id,branch_id,kind,fact_id),
    FOREIGN KEY(world_id,revision,branch_id) REFERENCES author_revisions(world_id,revision,branch_id)
) STRICT;
CREATE INDEX IF NOT EXISTS metadata_withdrawal_fact ON metadata_withdrawals(world_id,kind,fact_id,branch_id,revision);
CREATE TRIGGER IF NOT EXISTS metadata_withdrawal_validate BEFORE INSERT ON metadata_withdrawals
WHEN (new.kind='network' AND NOT EXISTS(SELECT 1 FROM network_facts WHERE world_id=new.world_id AND id=new.fact_id))
  OR (new.kind='parentage' AND NOT EXISTS(SELECT 1 FROM accepted_parentage WHERE world_id=new.world_id AND id=new.fact_id))
BEGIN SELECT RAISE(ABORT,'unknown metadata fact'); END;
