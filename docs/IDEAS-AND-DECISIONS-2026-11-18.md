# SemanticAnt: where the idea came from, and what is decided (captured 2026-11-18 by the portfolio PM)

## What CireSnave said
1. CireSnave, 2026-11-18, verbatim: "AnalysAnt's multiple lenses are what spawned the idea for the other but they are likely to be tool sets that AnalysAnt's lenses use rather than the same thing as AnalysAnt's lenses. EpistemologicAnt (or maybe EpistemAnt?...the names aren't set in stone), OntologAnt, and SemanticAnt (or maybe SemAnt?) are concepts for comprehensive libraries of tools. For example, EpistemologicAnt would include comprehensive tools libraries covering the entire field of epistemology and epistemological analysis."
2. CireSnave, 2026-11-18, verbatim: "EpistemologicAnt, OntologicAnt, and SemanticAnt can be the names. Go ahead and reserve them, create lanes for them, and get the basic idea into a spec for each. If something needs a tool that falls in one of those fields, we should build that tool in those projects and use it as a dependency for the project that needs it."

## The standing rule (CireSnave, 2026-11-18)
If something needs a tool that falls in one of these fields (epistemology, ontology, semantics), the tool is built in the matching project and
used as a dependency by the project that needs it. A consumer (AnalysAnt, SageAnt, others) does not grow its own copy.

## Decided
- Tools here are generic over any use of that tool type and not specific to an individual consumer (CireSnave 2026-11-18).
- The name is SemanticAnt (crate `semanticant`; placeholder 0.0.0 published). Names were "not set in stone" on 2026-11-17 and were fixed on 2026-11-18.
- A lane exists for this project (created asleep; started by the PM).
- Layering: field libraries (this project and its two siblings) <- AnalysAnt's lenses <- MCP tools <- editing tools. SageAnt may use them too.

## Not decided
Everything else: scope, the tool list, the crate split, the API. See `SPEC-SEED-2026-11-18.md` (the PM's seed) and the spec the lane writes.
