use crate::PackError;
use semanticant_core::pack::{FileEntry, Manifest, Sha256Hex};
use semanticant_core::{LangTag, Provenance};
use toml::{Table, Value};

fn str_of<'a>(t: &'a Table, k: &'static str) -> Result<&'a str, PackError> {
    t.get(k).and_then(Value::as_str).ok_or(PackError::Field(k))
}
fn opt_str(t: &Table, k: &'static str) -> Result<Option<String>, PackError> {
    match t.get(k) {
        None => Ok(None),
        Some(v) => v
            .as_str()
            .map(|s| Some(s.to_string()))
            .ok_or(PackError::Field(k)),
    }
}

/// Parse and VALIDATE a `pack.toml`. Unknown keys are ignored (forward compatibility within a schema).
pub fn parse_manifest(text: &str) -> Result<Manifest, PackError> {
    let t: Table = text
        .parse()
        .map_err(|e: toml::de::Error| PackError::Toml(e.to_string()))?;
    let schema = t
        .get("schema")
        .and_then(Value::as_integer)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or(PackError::Field("schema"))?;
    let prov = t
        .get("provenance")
        .and_then(Value::as_table)
        .ok_or(PackError::Field("provenance"))?;
    let files = t
        .get("files")
        .and_then(Value::as_array)
        .ok_or(PackError::Field("files"))?;
    let mut entries = Vec::new();
    for f in files {
        let f = f.as_table().ok_or(PackError::Field("files"))?;
        entries.push(FileEntry {
            name: str_of(f, "name")?.to_string(),
            sha256: Sha256Hex::parse(str_of(f, "sha256")?)?,
        });
    }
    let m = Manifest {
        name: str_of(&t, "name")?.to_string(),
        version: str_of(&t, "version")?.to_string(),
        kind: str_of(&t, "kind")?.to_string(),
        schema,
        lang: opt_str(&t, "lang")?
            .map(|l| LangTag::parse(&l))
            .transpose()?,
        provenance: Provenance {
            source: str_of(prov, "source")?.to_string(),
            licence: str_of(prov, "licence")?.to_string(),
            credit: str_of(prov, "credit")?.to_string(),
            url: opt_str(prov, "url")?,
            retrieved: opt_str(prov, "retrieved")?,
        },
        files: entries,
    };
    m.validate()?;
    Ok(m)
}

/// Render a manifest as `pack.toml` text (for pack-building tools). Does not validate.
pub fn manifest_to_toml(m: &Manifest) -> String {
    let mut t = Table::new();
    t.insert("schema".into(), Value::Integer(i64::from(m.schema)));
    t.insert("name".into(), Value::String(m.name.clone()));
    t.insert("version".into(), Value::String(m.version.clone()));
    t.insert("kind".into(), Value::String(m.kind.clone()));
    if let Some(l) = &m.lang {
        t.insert("lang".into(), Value::String(l.to_string()));
    }
    let mut p = Table::new();
    p.insert("source".into(), Value::String(m.provenance.source.clone()));
    p.insert(
        "licence".into(),
        Value::String(m.provenance.licence.clone()),
    );
    p.insert("credit".into(), Value::String(m.provenance.credit.clone()));
    if let Some(u) = &m.provenance.url {
        p.insert("url".into(), Value::String(u.clone()));
    }
    if let Some(r) = &m.provenance.retrieved {
        p.insert("retrieved".into(), Value::String(r.clone()));
    }
    t.insert("provenance".into(), Value::Table(p));
    let files = m
        .files
        .iter()
        .map(|f| {
            let mut e = Table::new();
            e.insert("name".into(), Value::String(f.name.clone()));
            e.insert(
                "sha256".into(),
                Value::String(f.sha256.as_str().to_string()),
            );
            Value::Table(e)
        })
        .collect();
    t.insert("files".into(), Value::Array(files));
    t.to_string()
}
