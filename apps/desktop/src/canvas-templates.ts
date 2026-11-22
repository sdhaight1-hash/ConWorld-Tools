import type { Block } from "./entry-editor";

// Immutable built-in v1 starting structure. All inserted data is presentation-only;
// no canonical values are copied, and all factual queries begin as author-only.
export const templates = [
  { key: "blank", label: "Blank entry", sections: ["Overview"] },
  { key: "character", label: "Character profile", sections: ["Biography", "Relationships", "Notes"] },
  { key: "family", label: "Family / noble house", sections: ["House history", "Lineage", "Holdings"] },
  { key: "organization", label: "Organization", sections: ["Purpose", "Membership", "History"] },
  { key: "event", label: "Historical event", sections: ["Overview", "Participants", "Consequences"] },
  { key: "conflict", label: "Conflict", sections: ["Background", "Sides", "Outcome"] },
  { key: "location", label: "Location", sections: ["Geography", "Settlements", "History"] },
  { key: "genealogy", label: "Genealogy report", sections: ["Family origins", "Descent", "Evidence"] },
  { key: "source", label: "Historical source", sections: ["Description", "Provenance", "Extracts"] },
] as const;

function block(content: Block["content"]): Block {
  return { id: crypto.randomUUID(), audience: "Author", content };
}

export function historicalComponent(kind: string, world: string, id: string): Block | undefined {
  const target = { world, id };
  const role = ({ actors: "Actor", witnesses: "Witness", victims: "Victim",
    stakeholders: "Stakeholder" } as Record<string, string>)[kind];
  const filter = kind === "person_events" ? { ParticipantEvents: { participant: target, role: null } }
    : role ? { EventRole: { event: target, role } } : undefined;
  if (!filter) return undefined;
  return block({ Query: { schema_version: 1, filter, limit: 50 } });
}

export function instantiateTemplate(key: string, world: string, entryId: string): Block | undefined {
  const template = templates.find(item => item.key === key);
  if (!template) return undefined;
  const sections = template.sections.map(label => block({ Panel: [
    block({ Text: label }), block({ Text: "" }),
  ] }));
  const query = key === "character" ? historicalComponent("person_events", world, entryId)
    : key === "event" ? historicalComponent("actors", world, entryId) : undefined;
  if (query) sections.push(query);
  return block({ Template: { key, version: 1, children: [
    block({ Grid: { columns: 2, children: sections } }),
  ] } });
}
