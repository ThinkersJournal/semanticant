use crate::rules::{CueDef, CueKind, CueRules, Direction};
use semanticant_core::{Reason, Span, Verdict};
use semanticant_text::{Token, TokenKind};

/// A cue found in text.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    /// What it expresses.
    pub kind: CueKind,
    /// The cue phrase in the original text.
    pub span: Span,
    /// What it applies to; `None` for a cue with no direction or an empty scope.
    pub scope: Option<Span>,
    /// 0..=1, from the data.
    pub strength: f32,
    /// True if a pseudo-cue phrase covers it: it should NOT be treated as a real cue.
    pub suppressed: bool,
}

/// Find cues in `tokens` (as produced by `semanticant-text`; whitespace tokens are ignored). At each position the longest
/// matching cue phrase wins and its tokens are consumed. Suppressed cues are still returned, flagged, so the suppression is auditable.
pub fn find_cues(tokens: &[Token], rules: &CueRules) -> Vec<Verdict<Cue>> {
    let content: Vec<&Token> = tokens
        .iter()
        .filter(|t| t.kind != TokenKind::Whitespace)
        .collect();
    let low: Vec<String> = content.iter().map(|t| t.text.to_lowercase()).collect();
    let matches_at = |words: &[String], at: usize| {
        at + words.len() <= low.len() && low[at..at + words.len()] == *words
    };
    let mut out = Vec::new();
    let mut p = 0;
    while p < content.len() {
        let best: Option<&CueDef> = rules
            .cues
            .iter()
            .filter(|c| matches_at(&c.words, p))
            .max_by_key(|c| c.words.len());
        let Some(def) = best else {
            p += 1;
            continue;
        };
        let l = def.words.len();
        let span =
            Span::new(content[p].span.start(), content[p + l - 1].span.end()).expect("ordered");
        let mut reasons = vec![Reason::at(
            "cues.cue.match",
            span,
            format!("cue phrase {:?}", def.words.join(" ")),
        )];
        // pseudo-cues: any pseudo phrase that starts at or before p and ends at or after the cue's end
        let covering = rules.pseudo.iter().find_map(|ph| {
            (p.saturating_sub(ph.len() - 1)..=p)
                .find(|&q| matches_at(ph, q) && q + ph.len() >= p + l)
                .map(|q| {
                    Span::new(
                        content[q].span.start(),
                        content[q + ph.len() - 1].span.end(),
                    )
                    .expect("ordered")
                })
        });
        let suppressed = covering.is_some();
        if let Some(c) = covering {
            reasons.push(Reason::at(
                "cues.cue.suppressed_pseudo",
                c,
                "inside a pseudo-cue phrase",
            ));
        }
        let scope = match def.direction {
            Direction::None => None,
            Direction::Forward => scan(&content, &low, rules, p + l, 1, def.window, &mut reasons),
            Direction::Backward => {
                if p == 0 {
                    reasons.push(Reason::new(
                        "cues.scope.text_edge",
                        "no tokens before the cue",
                    ));
                    None
                } else {
                    scan(&content, &low, rules, p - 1, -1, def.window, &mut reasons)
                }
            }
        };
        out.push(Verdict {
            value: Cue {
                kind: def.kind.clone(),
                span,
                scope,
                strength: def.strength,
                suppressed,
            },
            reasons,
        });
        p += l;
    }
    out
}

/// Walk from content index `from` in direction `step`, taking up to `window` tokens, stopping at a terminator or a sentence end.
fn scan(
    content: &[&Token],
    low: &[String],
    rules: &CueRules,
    from: usize,
    step: i64,
    window: usize,
    reasons: &mut Vec<Reason>,
) -> Option<Span> {
    let mut taken: Vec<usize> = Vec::new();
    let mut idx = from as i64;
    let stop = loop {
        if idx < 0 || idx as usize >= content.len() {
            break ("cues.scope.text_edge", "scope ran to the edge of the text");
        }
        let i = idx as usize;
        if rules.ends.contains(&content[i].text) {
            break (
                "cues.scope.sentence_end",
                "scope stopped at a sentence-end token",
            );
        }
        if rules.terminators.contains(&low[i]) {
            break (
                "cues.scope.terminator",
                "scope stopped at a terminator word",
            );
        }
        if taken.len() == window {
            break ("cues.scope.window", "scope reached the window size");
        }
        taken.push(i);
        idx += step;
    };
    let scope = match (taken.first(), taken.last()) {
        (Some(&a), Some(&b)) => {
            let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
            Some(Span::new(content[lo].span.start(), content[hi].span.end()).expect("ordered"))
        }
        _ => None,
    };
    match scope {
        Some(s) => reasons.push(Reason::at(stop.0, s, stop.1)),
        None => reasons.push(Reason::new(
            "cues.scope.empty",
            format!("empty scope: {}", stop.1),
        )),
    }
    scope
}
