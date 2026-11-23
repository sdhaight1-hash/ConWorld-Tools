-- Index source and descriptor backlinks without materializing a second graph.
CREATE INDEX IF NOT EXISTS network_source ON network_facts(world_id,source_id,recorded_revision);
CREATE INDEX IF NOT EXISTS classification_source ON concept_classifications(world_id,source_id,recorded_revision);
CREATE INDEX IF NOT EXISTS classification_descriptor ON concept_classifications(world_id,descriptor_id,recorded_revision);
