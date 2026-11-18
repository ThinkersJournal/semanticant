# Reason rule ids

`Reason::rule` ids are stable: a consumer may match on them. Notes (`Reason::note`) are English, for humans, and may change.
Convention: dotted lowercase, `<crate-area>.<topic>.<detail>`. Each tool crate documents its ids in its own `RULES.md`; this crate emits none.
