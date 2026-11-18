# Reason rule ids (stable)

| id | emitted by | meaning |
|---|---|---|
| `text.sentence.uax29` | `UaxSegmenter::sentences` | span is a UAX #29 sentence, no adjustment |
| `text.sentence.abbreviation` | `UaxSegmenter::sentences` | a UAX #29 break after a listed abbreviation was removed (the reason's span is the abbreviation) |
