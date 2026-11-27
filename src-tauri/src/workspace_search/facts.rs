//! Deterministic, source-exact structured facts extracted from indexed passages.
//!
//! These records are candidates, not semantic verification. Only explicit,
//! labeled values are stored. The original quote, source identity, version
//! hash, location, and extraction method travel with every record.

use regex::Regex;
use rusqlite::{params, types::Value, Connection};
use std::sync::OnceLock;

const MAX_FACTS_PER_PASSAGE: usize = 32;
const MAX_EVIDENCE_CHARS: usize = 280;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredFact {
    pub subject: String,
    pub attribute: String,
    pub value: String,
    pub unit: Option<String>,
    pub conditions: Option<String>,
    pub evidence: String,
    pub location: String,
    pub extraction_method: String,
    pub normalized_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactRecord {
    pub kind: String,
    pub ref_id: String,
    pub sub_ref: Option<String>,
    pub title: String,
    pub subject: String,
    pub attribute: String,
    pub value: String,
    pub unit: Option<String>,
    pub conditions: Option<String>,
    pub location: String,
    pub evidence: String,
    pub document_version: String,
    pub verification_status: String,
    pub extraction_method: String,
    pub normalized_value: String,
}

/// Candidate fields with a clear label and a number, ISO date, or supported text value.
///
/// The label vocabulary deliberately stays finite. Guessing an attribute from
/// arbitrary prose would make a cache look more authoritative than it is.
fn aliases() -> &'static [(&'static str, &'static str)] {
    &[
        ("operating pressure", "pressure"),
        ("delivery date", "delivery_date"),
        ("payment terms", "payment_terms"),
        ("payment term", "payment_terms"),
        ("model number", "model_number"),
        ("part number", "model_number"),
        ("equipment identifier", "equipment_id"),
        ("equipment id", "equipment_id"),
        ("serial number", "equipment_id"),
        ("supplier name", "supplier"),
        ("manufacturer", "manufacturer"),
        ("supplier", "supplier"),
        ("vendor", "supplier"),
        ("technical requirement", "technical_requirement"),
        ("contractual obligation", "obligation"),
        ("obligation", "obligation"),
        ("material", "material"),
        ("party", "party"),
        ("acronym", "acronym"),
        ("expiration date", "expiration_date"),
        ("interest rate", "interest_rate"),
        ("fecha de entrega", "delivery_date"),
        ("fecha de vencimiento", "deadline"),
        ("precio", "price"),
        ("presión", "pressure"),
        ("presion", "pressure"),
        ("temperatura", "temperature"),
        ("cantidad", "quantity"),
        ("価格", "price"),
        ("金額", "amount"),
        ("数量", "quantity"),
        ("圧力", "pressure"),
        ("温度", "temperature"),
        ("寸法", "dimensions"),
        ("納期", "delivery_date"),
        ("价格", "price"),
        ("金额", "amount"),
        ("数量", "quantity"),
        ("压力", "pressure"),
        ("温度", "temperature"),
        ("尺寸", "dimensions"),
        ("交付日期", "delivery_date"),
        ("供应商名称", "supplier"),
        ("供应商", "supplier"),
        ("型号", "model_number"),
        ("设备编号", "equipment_id"),
        ("材质", "material"),
        ("供給元", "supplier"),
        ("仕入先", "supplier"),
        ("型番", "model_number"),
        ("製造番号", "equipment_id"),
        ("proveedor", "supplier"),
        ("fabricante", "manufacturer"),
        ("número de modelo", "model_number"),
        ("material", "material"),
        ("obligación", "obligation"),
        ("requisito técnico", "technical_requirement"),
        ("budget", "budget"),
        ("cost", "cost"),
        ("price", "price"),
        ("amount", "amount"),
        ("quantity", "quantity"),
        ("qty", "quantity"),
        ("pressure", "pressure"),
        ("temperature", "temperature"),
        ("temp", "temperature"),
        ("dimensions", "dimensions"),
        ("dimension", "dimensions"),
        ("length", "length"),
        ("width", "width"),
        ("height", "height"),
        ("depth", "depth"),
        ("diameter", "diameter"),
        ("thickness", "thickness"),
        ("weight", "weight"),
        ("mass", "weight"),
        ("volume", "volume"),
        ("date", "date"),
        ("deadline", "deadline"),
        ("due date", "deadline"),
        ("start date", "start_date"),
        ("end date", "end_date"),
        ("delivery", "delivery_date"),
        ("term", "term"),
        ("payment", "payment_terms"),
        ("rate", "rate"),
        ("capacity", "capacity"),
        ("voltage", "voltage"),
        ("power", "power"),
        ("speed", "speed"),
        ("duration", "duration"),
        ("milestone", "milestone"),
    ]
}

fn field_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(concat!(
            r"(?iu)(?P<label>[\p{L}][\p{L}\p{N} _./-]{0,63})\s*[:=]\s*",
            r"(?P<value>(?:(?:USD|EUR|GBP|QAR|SAR|AED|JPY|CNY|CAD|AUD)\s*)?",
            r"(?:[$€£¥]\s*)?[+-]?(?:\d{4}-\d{2}-\d{2}|\d[\d,]*(?:\.\d+)?)",
            r"(?:\s*(?:%|(?:percent|USD|EUR|GBP|QAR|SAR|AED|JPY|CNY|CAD|AUD|",
            r"mm|cm|km|in|ft|m|kg|mg|g|lb|lbs|tonnes?|psi|kPa|MPa|bar|",
            r"°C|°F|Celsius|Fahrenheit|V|kV|W|kW|MW|kWh|L|litres?|liters?|",
            r"m3|m³|units?|pieces?|pcs|days?|weeks?|months?|years?|hours?|hrs?|",
            r"business days?)\b))?)"
        ))
        .expect("structured fact pattern is valid")
    })
}

fn text_field_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?imu)^[ \t]*(?P<label>[\p{L}][\p{L}\p{N} _./-]{0,63})\s*[:=]\s*(?P<value>[^\r\n]{1,140})[ \t]*$",
        )
        .expect("structured text fact pattern is valid")
    })
}

fn is_text_attribute(attribute: &str) -> bool {
    matches!(
        attribute,
        "model_number"
            | "equipment_id"
            | "supplier"
            | "manufacturer"
            | "technical_requirement"
            | "obligation"
            | "payment_terms"
            | "material"
            | "party"
            | "acronym"
    )
}

fn canonical_attribute(label: &str) -> Option<(String, String)> {
    let clean = label.trim().trim_matches(['-', '/', '.']).trim();
    let lower = clean.to_lowercase();
    // Longest suffix wins, allowing labels such as "Atlas Project Budget".
    let mut matches: Vec<_> = aliases()
        .iter()
        .filter(|(alias, _)| {
            lower == *alias
                || lower.strip_suffix(alias).is_some_and(|prefix| {
                    prefix.ends_with(' ') || alias.chars().all(|c| !c.is_ascii_alphanumeric())
                })
        })
        .collect();
    matches.sort_by_key(|(alias, _)| std::cmp::Reverse(alias.chars().count()));
    let (alias, canonical) = matches.first()?;
    let subject = lower
        .strip_suffix(alias)
        .unwrap_or("")
        .trim_matches([' ', '-', '/', '.'])
        .to_string();
    Some((canonical.to_string(), subject))
}

fn unit_from(raw: &str) -> Option<(String, String)> {
    let lower = raw.to_lowercase();
    let currencies = [
        ("$", "USD"),
        ("€", "EUR"),
        ("£", "GBP"),
        ("¥", "JPY"),
        ("usd", "USD"),
        ("eur", "EUR"),
        ("gbp", "GBP"),
        ("qar", "QAR"),
        ("sar", "SAR"),
        ("aed", "AED"),
        ("jpy", "JPY"),
        ("cny", "CNY"),
        ("cad", "CAD"),
        ("aud", "AUD"),
    ];
    for (needle, code) in currencies {
        if lower.contains(needle) {
            return Some((code.to_string(), code.to_lowercase()));
        }
    }
    let units = [
        ("business days", "business_day"),
        ("fahrenheit", "°F"),
        ("celsius", "°C"),
        ("tonnes", "tonne"),
        ("tonne", "tonne"),
        ("months", "month"),
        ("weeks", "week"),
        ("years", "year"),
        ("hours", "hour"),
        ("litres", "L"),
        ("liters", "L"),
        ("pieces", "piece"),
        ("units", "unit"),
        ("days", "day"),
        ("months", "month"),
        ("weeks", "week"),
        ("years", "year"),
        ("hours", "hour"),
        ("hrs", "hour"),
        ("m³", "m3"),
        ("m3", "m3"),
        ("kwh", "kWh"),
        ("mpa", "MPa"),
        ("kpa", "kPa"),
        ("kw", "kW"),
        ("kv", "kV"),
        ("mw", "MW"),
        ("kg", "kg"),
        ("mg", "mg"),
        ("lb", "lb"),
        ("lbs", "lb"),
        ("mm", "mm"),
        ("cm", "cm"),
        ("km", "km"),
        ("psi", "psi"),
        ("bar", "bar"),
        ("°c", "°C"),
        ("°f", "°F"),
        ("%", "%"),
        ("ft", "ft"),
        ("in", "in"),
        ("kg", "kg"),
        ("kwh", "kWh"),
        ("kw", "kW"),
        ("kv", "kV"),
        ("mw", "MW"),
        ("l", "L"),
        ("m", "m"),
        ("g", "g"),
        ("v", "V"),
        ("w", "W"),
    ];
    for (needle, unit) in units {
        if lower
            .split_whitespace()
            .any(|part| part.trim_matches(['.', ',']) == needle)
            || lower.ends_with(needle)
        {
            return Some((unit.to_string(), unit.to_lowercase()));
        }
    }
    None
}

fn normalized_number(raw: &str) -> Option<String> {
    static DATE: OnceLock<Regex> = OnceLock::new();
    let date = DATE.get_or_init(|| Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap());
    if date.is_match(raw.trim()) {
        return Some(raw.trim().to_string());
    }
    static NUMBER: OnceLock<Regex> = OnceLock::new();
    let number = NUMBER.get_or_init(|| Regex::new(r"[+-]?(?:\d[\d,]*(?:\.\d+)?)").unwrap());
    let captured = number.find(raw)?.as_str().replace(',', "");
    let (sign, unsigned) = match captured.chars().next()? {
        '-' => ("-", &captured[1..]),
        '+' => ("", &captured[1..]),
        _ => ("", captured.as_str()),
    };
    let (integer, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };
    let fractional = fractional.trim_end_matches('0');
    if fractional.is_empty() {
        Some(format!("{sign}{integer}"))
    } else {
        Some(format!("{sign}{integer}.{fractional}"))
    }
}

fn sentence_for(text: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let before = &text[..start];
    let after = &text[end..];
    let left = before
        .char_indices()
        .rev()
        .find(|(_, c)| matches!(c, '\n' | ';' | '.' | '!' | '?' | '。' | '！' | '？'))
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    let right = after
        .char_indices()
        .find(|(_, c)| matches!(c, '\n' | ';' | '.' | '!' | '?' | '。' | '！' | '？'))
        .map(|(i, _)| end + i)
        .unwrap_or(text.len());
    let trimmed = text[left..right].trim();
    let offset = text[left..right].find(trimmed)?;
    Some((left + offset, left + offset + trimmed.len()))
}

/// Extract explicit, labeled values from one search passage.
pub fn extract(text: &str, title: &str, location: &str) -> Vec<StructuredFact> {
    let mut facts = Vec::new();
    for caps in field_re().captures_iter(text) {
        let Some(m) = caps.get(0) else { continue };
        let Some((attribute, label_subject)) = canonical_attribute(&caps["label"]) else {
            continue;
        };
        let raw_value = caps["value"].trim();
        let Some(number) = normalized_number(raw_value) else {
            continue;
        };
        let is_date = Regex::new(r"^\d{4}-\d{2}-\d{2}$")
            .unwrap()
            .is_match(raw_value);
        let (unit, unit_key) = if is_date {
            (Some("date".to_string()), "date".to_string())
        } else {
            unit_from(raw_value).map_or((None, String::new()), |(u, k)| (Some(u), k))
        };
        let Some((q_start, q_end)) = sentence_for(text, m.start(), m.end()) else {
            continue;
        };
        let evidence = text[q_start..q_end].trim().to_string();
        if evidence.chars().count() > MAX_EVIDENCE_CHARS || evidence.is_empty() {
            continue;
        }
        let trailing = text[m.end()..q_end]
            .trim()
            .trim_matches(['.', ',', ';', ':']);
        let conditions = (!trailing.is_empty()).then(|| trailing.to_string());
        let subject = if label_subject.is_empty() {
            title.trim().to_string()
        } else {
            label_subject
        };
        let normalized_value = format!("{number}|{unit_key}");
        let candidate = StructuredFact {
            subject,
            attribute,
            value: raw_value.to_string(),
            unit,
            conditions,
            evidence,
            location: location.to_string(),
            extraction_method: "labeled_numeric_v1".into(),
            normalized_value,
        };
        if !facts.iter().any(|f: &StructuredFact| {
            f.attribute == candidate.attribute
                && f.subject.eq_ignore_ascii_case(&candidate.subject)
                && f.normalized_value == candidate.normalized_value
                && f.evidence == candidate.evidence
        }) {
            facts.push(candidate);
        }
        if facts.len() >= MAX_FACTS_PER_PASSAGE {
            break;
        }
    }
    if facts.len() < MAX_FACTS_PER_PASSAGE {
        for caps in text_field_re().captures_iter(text) {
            let Some(m) = caps.get(0) else { continue };
            let Some((attribute, label_subject)) = canonical_attribute(&caps["label"]) else {
                continue;
            };
            if !is_text_attribute(&attribute) {
                continue;
            }
            let value = caps["value"].trim();
            if value.is_empty() || value.chars().count() > 140 {
                continue;
            }
            let evidence = m.as_str().trim().to_string();
            if evidence.chars().count() > MAX_EVIDENCE_CHARS {
                continue;
            }
            let subject = if label_subject.is_empty() {
                title.trim().to_string()
            } else {
                label_subject
            };
            let normalized_value = value
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase();
            let candidate = StructuredFact {
                subject,
                attribute,
                value: value.to_string(),
                unit: None,
                conditions: None,
                evidence,
                location: location.to_string(),
                extraction_method: "labeled_text_v1".into(),
                normalized_value,
            };
            if !facts.iter().any(|f: &StructuredFact| {
                f.attribute == candidate.attribute
                    && f.subject.eq_ignore_ascii_case(&candidate.subject)
                    && f.normalized_value == candidate.normalized_value
                    && f.evidence == candidate.evidence
            }) {
                facts.push(candidate);
            }
            if facts.len() >= MAX_FACTS_PER_PASSAGE {
                break;
            }
        }
    }
    facts
}

pub fn schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS source_facts (
            id INTEGER PRIMARY KEY,
            kind TEXT NOT NULL CHECK(kind IN ('transcript','document','folder','talking_point')),
            ref_id TEXT NOT NULL,
            sub_ref TEXT,
            title TEXT NOT NULL DEFAULT '',
            subject TEXT NOT NULL,
            attribute TEXT NOT NULL,
            value TEXT NOT NULL,
            unit TEXT,
            conditions TEXT,
            source_location TEXT NOT NULL DEFAULT '',
            supporting_text TEXT NOT NULL,
            document_version TEXT NOT NULL,
            verification_status TEXT NOT NULL CHECK(verification_status = 'source_exact'),
            extraction_method TEXT NOT NULL,
            normalized_value TEXT NOT NULL,
            extractor_version INTEGER NOT NULL DEFAULT 1
        );
        CREATE INDEX IF NOT EXISTS idx_source_facts_lookup
            ON source_facts(attribute, subject, kind, ref_id);
        CREATE INDEX IF NOT EXISTS idx_source_facts_source
            ON source_facts(kind, ref_id, sub_ref);
        CREATE INDEX IF NOT EXISTS idx_source_facts_conflicts
            ON source_facts(subject, attribute, normalized_value);",
    )
}

/// Migration v20: backfill the cache from the already indexed, source-bound
/// passages. Existing documents therefore do not need to be reimported.
pub fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT p.kind, p.ref_id, p.sub_ref, p.title, p.start_ms, p.end_ms,
                p.text, p.tokens, p.occurred_at, p.cjk, s.content_hash
         FROM search_passages p
         JOIN search_sources s ON s.kind = p.kind AND s.ref_id = p.ref_id
             AND s.sub_ref = COALESCE(p.sub_ref, '')
         WHERE s.status = 'ok'
         ORDER BY p.id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, Option<i64>>(4)?,
            r.get::<_, Option<i64>>(5)?,
            r.get::<_, String>(6)?,
            r.get::<_, i64>(7)?,
            r.get::<_, Option<i64>>(8)?,
            r.get::<_, i64>(9)? != 0,
            r.get::<_, String>(10)?,
        ))
    })?;
    let mut grouped: std::collections::BTreeMap<
        (String, String, Option<String>, String),
        Vec<super::passages::NewPassage>,
    > = std::collections::BTreeMap::new();
    for row in rows {
        let (kind, ref_id, sub_ref, title, start_ms, end_ms, text, tokens, occurred_at, cjk, hash) =
            row?;
        let Some(kind) = super::PassageKind::parse(&kind) else {
            continue;
        };
        grouped
            .entry((
                kind.as_str().to_string(),
                ref_id.clone(),
                sub_ref.clone(),
                hash,
            ))
            .or_default()
            .push(super::passages::NewPassage {
                kind,
                ref_id,
                sub_ref,
                title,
                start_ms,
                end_ms,
                text,
                tokens: tokens.max(0) as usize,
                occurred_at,
                cjk,
            });
    }
    for ((kind, ref_id, sub_ref, version), passages) in grouped {
        let Some(kind) = super::PassageKind::parse(&kind) else {
            continue;
        };
        replace_for_passages(conn, kind, &ref_id, sub_ref.as_deref(), &passages, &version)?;
    }
    Ok(())
}

/// Replace the facts for one indexed source part in the same transaction as
/// the passages. A changed source version cannot leave stale values behind.
pub fn replace_for_passages(
    conn: &Connection,
    kind: super::PassageKind,
    ref_id: &str,
    sub_ref: Option<&str>,
    passages: &[super::passages::NewPassage],
    version: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM source_facts WHERE kind = ?1 AND ref_id = ?2 AND sub_ref IS ?3",
        params![kind.as_str(), ref_id, sub_ref],
    )?;
    let mut insert = conn.prepare_cached(
        "INSERT INTO source_facts
         (kind, ref_id, sub_ref, title, subject, attribute, value, unit, conditions,
          source_location, supporting_text, document_version, verification_status,
          extraction_method, normalized_value, extractor_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                 'source_exact', ?13, ?14, 1)",
    )?;
    for passage in passages {
        let location =
            passage
                .sub_ref
                .clone()
                .unwrap_or_else(|| match (passage.start_ms, passage.end_ms) {
                    (Some(start), Some(end)) => format!("time:{start}-{end}"),
                    (Some(start), None) => format!("time:{start}"),
                    _ => String::new(),
                });
        for fact in extract(&passage.text, &passage.title, &location) {
            insert.execute(params![
                passage.kind.as_str(),
                passage.ref_id,
                passage.sub_ref,
                passage.title,
                fact.subject,
                fact.attribute,
                fact.value,
                fact.unit,
                fact.conditions,
                fact.location,
                fact.evidence,
                version,
                fact.extraction_method,
                fact.normalized_value
            ])?;
        }
    }
    Ok(())
}

/// Find an attribute explicitly named in a question. This is a retrieval hint,
/// never a claim that the question has a unique or verified answer.
pub fn question_attribute(question: &str) -> Option<String> {
    let lower = question.to_lowercase();
    aliases()
        .iter()
        .filter(|(alias, _)| {
            if alias.chars().any(|c| !c.is_ascii_alphanumeric()) {
                lower.contains(alias)
            } else {
                lower
                    .split(|c: char| !c.is_alphanumeric())
                    .any(|w| w == *alias)
            }
        })
        .max_by_key(|(alias, _)| alias.chars().count())
        .map(|(_, canonical)| canonical.to_string())
}

/// A deliberately small set of question forms that asks for one cached value.
/// Comparative, explanatory, and open-ended questions fall through to search
/// and generation even when they mention a supported attribute.
fn is_direct_lookup(question: &str) -> bool {
    let lower = question.trim().to_lowercase();
    if lower.matches('?').count() > 1
        || [
            "compare",
            "difference",
            "trend",
            "explain",
            "why",
            "between",
            "versus",
            " vs ",
            " and how ",
            " and what ",
            " or what ",
        ]
        .iter()
        .any(|term| lower.contains(term))
    {
        return false;
    }
    [
        "what is ",
        "what's ",
        "what was ",
        "what will be ",
        "how much ",
        "how many ",
        "when is ",
        "when was ",
        "when will ",
        "what date ",
        "what amount ",
        "cuál es ",
        "cual es ",
        "cuánto es ",
        "cuanto es ",
        "cuándo es ",
        "cuando es ",
    ]
    .iter()
    .any(|prefix| lower.starts_with(prefix))
        || ["いくら", "いつ", "何日", "はいくつ", "是什么", "是多少"]
            .iter()
            .any(|marker| lower.contains(marker))
}

/// Return a source-exact candidate only for a precise lookup with one
/// unambiguous subject, value, condition set, and source. A cache candidate is
/// never treated as a semantic answer to a broader question.
pub fn direct_lookup<'a>(question: &str, records: &'a [FactRecord]) -> Option<&'a FactRecord> {
    let attribute = question_attribute(question)?;
    if !is_direct_lookup(question) || records.is_empty() || has_conflict(records) {
        return None;
    }
    let first = records.first()?;
    if first.attribute != attribute
        || first.verification_status != "source_exact"
        || first.extraction_method != "labeled_numeric_v1"
        || first.document_version.is_empty()
    {
        return None;
    }
    let same_fact = records.iter().all(|record| {
        record.attribute == first.attribute
            && record.subject.eq_ignore_ascii_case(&first.subject)
            && record.conditions.as_deref().unwrap_or("")
                == first.conditions.as_deref().unwrap_or("")
            && record.normalized_value == first.normalized_value
            && record.verification_status == "source_exact"
            && record.extraction_method == "labeled_numeric_v1"
            && !record.document_version.is_empty()
    });
    let same_source = records.iter().all(|record| {
        record.kind == first.kind
            && record.ref_id == first.ref_id
            && record.sub_ref == first.sub_ref
    });
    (same_fact && same_source).then_some(first)
}

pub fn display_attribute(attribute: &str) -> &'static str {
    match attribute {
        "budget" => "budget",
        "cost" => "cost",
        "price" => "price",
        "amount" => "amount",
        "quantity" => "quantity",
        "pressure" => "pressure",
        "temperature" => "temperature",
        "dimensions" => "dimensions",
        "length" => "length",
        "width" => "width",
        "height" => "height",
        "depth" => "depth",
        "diameter" => "diameter",
        "thickness" => "thickness",
        "weight" => "weight",
        "volume" => "volume",
        "delivery_date" => "delivery date",
        "expiration_date" => "expiration date",
        "start_date" => "start date",
        "end_date" => "end date",
        "deadline" => "deadline",
        "payment_terms" => "payment terms",
        "interest_rate" => "interest rate",
        "model_number" => "model number",
        "equipment_id" => "equipment identifier",
        "supplier" => "supplier",
        "manufacturer" => "manufacturer",
        "technical_requirement" => "technical requirement",
        "obligation" => "obligation",
        "material" => "material",
        "party" => "party",
        "acronym" => "acronym",
        "term" => "term",
        "rate" => "rate",
        "capacity" => "capacity",
        "voltage" => "voltage",
        "power" => "power",
        "speed" => "speed",
        "duration" => "duration",
        "milestone" => "milestone",
        _ => "value",
    }
}

/// Scope-bound source facts for an explicitly named attribute. This returns
/// only facts from the same effective source set used by workspace search.
pub fn matching(
    conn: &Connection,
    scope: &super::SearchScope,
    question: &str,
    limit: usize,
) -> rusqlite::Result<Vec<FactRecord>> {
    let Some(attribute) = question_attribute(question) else {
        return Ok(Vec::new());
    };
    let mut alts = Vec::new();
    let mut values: Vec<Value> = Vec::new();
    match &scope.meetings {
        super::MeetingScope::None => {}
        super::MeetingScope::Ids(ids) if ids.is_empty() => {}
        super::MeetingScope::Ids(ids) => {
            alts.push(
                "(f.kind = 'transcript' AND f.ref_id IN (SELECT value FROM json_each(?)))"
                    .to_string(),
            );
            values.push(Value::Text(
                serde_json::to_string(ids).unwrap_or_else(|_| "[]".into()),
            ));
        }
        super::MeetingScope::HomeWorkspace(workspace) => {
            alts.push("(f.kind = 'transcript' AND f.ref_id IN (SELECT id FROM meetings WHERE workspace_id = ?))".to_string());
            values.push(Value::Text(workspace.clone()));
        }
    }
    for (kind, ids) in [
        ("document", &scope.document_ids),
        ("folder", &scope.folder_ids),
        ("talking_point", &scope.talking_point_ids),
    ] {
        if !ids.is_empty() {
            alts.push(format!(
                "(f.kind = '{kind}' AND f.ref_id IN (SELECT value FROM json_each(?)))"
            ));
            values.push(Value::Text(
                serde_json::to_string(ids).unwrap_or_else(|_| "[]".into()),
            ));
        }
    }
    if alts.is_empty() {
        return Ok(Vec::new());
    }
    let mut filter = format!("({})", alts.join(" OR "));
    if !scope.excluded_meeting_ids.is_empty() {
        filter.push_str(
            " AND NOT (f.kind = 'transcript' AND f.ref_id IN (SELECT value FROM json_each(?)))",
        );
        values.push(Value::Text(
            serde_json::to_string(&scope.excluded_meeting_ids).unwrap_or_else(|_| "[]".into()),
        ));
    }
    values.push(Value::Text(attribute));
    let attr_pos = values.len();
    values.push(Value::Integer(limit.clamp(1, 24) as i64));
    let limit_pos = values.len();
    let sql = format!(
        "SELECT f.kind, f.ref_id, f.sub_ref, f.title, f.subject, f.attribute, f.value,
                f.unit, f.conditions, f.source_location, f.supporting_text,
                f.document_version, f.verification_status, f.extraction_method,
                f.normalized_value
         FROM source_facts f WHERE {filter} AND f.attribute = ?{attr_pos}
         ORDER BY f.attribute, f.subject COLLATE NOCASE, f.id LIMIT ?{limit_pos}"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(values.iter()), |r| {
        Ok(FactRecord {
            kind: r.get(0)?,
            ref_id: r.get(1)?,
            sub_ref: r.get(2)?,
            title: r.get(3)?,
            subject: r.get(4)?,
            attribute: r.get(5)?,
            value: r.get(6)?,
            unit: r.get(7)?,
            conditions: r.get(8)?,
            location: r.get(9)?,
            evidence: r.get(10)?,
            document_version: r.get(11)?,
            verification_status: r.get(12)?,
            extraction_method: r.get(13)?,
            normalized_value: r.get(14)?,
        })
    })?;
    rows.collect()
}

/// True only when the same named subject and attribute has different values.
pub fn has_conflict(records: &[FactRecord]) -> bool {
    for (i, left) in records.iter().enumerate() {
        if records[i + 1..].iter().any(|right| {
            left.attribute == right.attribute
                && left.subject.eq_ignore_ascii_case(&right.subject)
                && left
                    .conditions
                    .as_deref()
                    .unwrap_or("")
                    .eq_ignore_ascii_case(right.conditions.as_deref().unwrap_or(""))
                && left.normalized_value != right.normalized_value
        }) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_labeled_value_with_exact_evidence_and_location_metadata() {
        let text = "Atlas Project Budget: QAR 1,250,000 for phase two.\nDelivery date: 2027-06-12.";
        let facts = extract(text, "Project Notes", "page:4");
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[0].subject, "atlas project");
        assert_eq!(facts[0].attribute, "budget");
        assert_eq!(facts[0].value, "QAR 1,250,000");
        assert_eq!(facts[0].unit.as_deref(), Some("QAR"));
        assert_eq!(facts[0].conditions.as_deref(), Some("for phase two"));
        assert_eq!(
            facts[0].evidence,
            "Atlas Project Budget: QAR 1,250,000 for phase two"
        );
        assert_eq!(facts[0].location, "page:4");
        assert_eq!(facts[1].attribute, "delivery_date");
        assert_eq!(facts[1].value, "2027-06-12");
        assert_eq!(facts[1].unit.as_deref(), Some("date"));
    }

    #[test]
    fn extracts_labeled_text_fields_without_guessing_from_prose() {
        let text = "Supplier name: Acme Industrial Ltd.\nModel number: RX-2108\nPayment terms: Net 30 days after delivery.\nMaterial: Stainless steel 316L.";
        let facts = extract(text, "Atlas Supply", "page:2");
        assert_eq!(facts.len(), 4);
        assert_eq!(facts[0].attribute, "supplier");
        assert_eq!(facts[0].value, "Acme Industrial Ltd.");
        assert_eq!(facts[0].extraction_method, "labeled_text_v1");
        assert_eq!(facts[0].evidence, "Supplier name: Acme Industrial Ltd.");
        assert_eq!(facts[1].attribute, "model_number");
        assert_eq!(facts[1].value, "RX-2108");
        assert_eq!(facts[2].attribute, "payment_terms");
        assert_eq!(facts[2].value, "Net 30 days after delivery.");
        assert_eq!(facts[3].attribute, "material");
        assert_eq!(facts[3].value, "Stainless steel 316L.");
        assert!(facts.iter().all(|fact| fact.unit.is_none()));
        assert!(extract("The supplier is Acme Industrial Ltd.", "Atlas", "page:1").is_empty());
    }

    #[test]
    fn requires_an_explicit_supported_label_and_preserves_distinct_values() {
        let facts = extract(
            "Some amount is 20 bar.\nPressure: 20 bar.\nPressure: 22 bar.",
            "Spec Sheet",
            "page:1",
        );
        assert_eq!(facts.len(), 2);
        assert_eq!(facts[0].attribute, "pressure");
        assert_ne!(facts[0].normalized_value, facts[1].normalized_value);
    }

    #[test]
    fn keeps_percent_units_and_large_decimal_values_exact() {
        let facts = extract(
            "Interest rate: 12.50%.\nCapacity: 12345678901234567890 units.",
            "Terms",
            "page:2",
        );
        assert_eq!(facts[0].unit.as_deref(), Some("%"));
        assert_eq!(facts[0].normalized_value, "12.5|%");
        assert_eq!(facts[1].normalized_value, "12345678901234567890|unit");
    }

    #[test]
    fn finds_multilingual_question_attributes_without_marking_them_verified() {
        assert_eq!(
            question_attribute("¿Cuál es el precio?"),
            Some("price".into())
        );
        assert_eq!(
            question_attribute("運転圧力はいくつですか"),
            Some("pressure".into())
        );
        assert_eq!(
            question_attribute("交付日期是什么？"),
            Some("delivery_date".into())
        );
    }

    #[test]
    fn direct_lookup_requires_one_precise_unambiguous_source_fact() {
        let fact = |subject: &str, value: &str, source: &str| FactRecord {
            kind: "document".into(),
            ref_id: source.into(),
            sub_ref: Some("page:1".into()),
            title: "Project Terms".into(),
            subject: subject.into(),
            attribute: "budget".into(),
            value: value.into(),
            unit: Some("QAR".into()),
            conditions: Some("phase two".into()),
            location: "page:1".into(),
            evidence: format!("{subject} budget: {value} for phase two"),
            document_version: "sha256:abc".into(),
            verification_status: "source_exact".into(),
            extraction_method: "labeled_numeric_v1".into(),
            normalized_value: format!("{value}|qar"),
        };
        let one = vec![fact("atlas", "QAR 50,000", "doc-a")];
        assert!(direct_lookup("What is the budget?", &one).is_some());
        assert!(direct_lookup("Explain the budget trend", &one).is_none());
        assert!(direct_lookup("What is the budget, and how did they justify it?", &one).is_none());

        let two_sources = vec![
            fact("atlas", "QAR 50,000", "doc-a"),
            fact("atlas", "QAR 50,000", "doc-b"),
        ];
        assert!(direct_lookup("What is the budget?", &two_sources).is_none());

        let two_subjects = vec![fact("atlas", "QAR 50,000", "doc-a"), {
            let mut other = fact("orion", "QAR 50,000", "doc-a");
            other.sub_ref = Some("page:2".into());
            other
        }];
        assert!(direct_lookup("What is the budget?", &two_subjects).is_none());

        let conflict = vec![fact("atlas", "QAR 50,000", "doc-a"), {
            let mut other = fact("atlas", "QAR 60,000", "doc-a");
            other.normalized_value = "60000|qar".into();
            other
        }];
        assert!(direct_lookup("What is the budget?", &conflict).is_none());
    }

    #[test]
    fn source_fact_schema_is_idempotent_and_marks_provenance_fields() {
        let conn = Connection::open_in_memory().unwrap();
        schema(&conn).unwrap();
        schema(&conn).unwrap();
        let cols: Vec<String> = conn
            .prepare("PRAGMA table_info(source_facts)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        for column in [
            "subject",
            "attribute",
            "value",
            "source_location",
            "supporting_text",
            "document_version",
            "verification_status",
            "extraction_method",
        ] {
            assert!(cols.iter().any(|c| c == column), "missing {column}");
        }
    }

    #[test]
    fn migration_backfills_source_exact_facts_from_existing_passages() {
        let conn = Connection::open_in_memory().unwrap();
        super::super::migrate::migrate_conn(&conn).unwrap();
        conn.execute(
            "INSERT INTO search_passages(kind, ref_id, title, text, tokens, cjk)
             VALUES ('document', 'doc-1', 'Atlas Spec', 'Atlas budget: QAR 50,000.', 8, 0)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO search_sources(kind, ref_id, sub_ref, content_hash, status)
             VALUES ('document', 'doc-1', '', 'source-hash', 'ok')",
            [],
        )
        .unwrap();
        migrate(&conn).unwrap();
        let row: (String, String, String, String) = conn
            .query_row(
                "SELECT attribute, document_version, supporting_text, extraction_method FROM source_facts WHERE ref_id = 'doc-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(row.0, "budget");
        assert_eq!(row.1, "source-hash");
        assert_eq!(row.2, "Atlas budget: QAR 50,000");
        assert_eq!(row.3, "labeled_numeric_v1");
    }

    #[test]
    fn replacing_or_emptying_a_source_invalidates_its_old_facts() {
        let conn = Connection::open_in_memory().unwrap();
        schema(&conn).unwrap();
        let passage = |text: &str| super::super::passages::NewPassage {
            kind: super::super::PassageKind::Document,
            ref_id: "doc-1".into(),
            sub_ref: Some("page:1".into()),
            title: "Atlas Spec".into(),
            start_ms: None,
            end_ms: None,
            text: text.into(),
            tokens: 8,
            occurred_at: None,
            cjk: false,
        };
        replace_for_passages(
            &conn,
            super::super::PassageKind::Document,
            "doc-1",
            Some("page:1"),
            &[passage("Budget: QAR 50,000.")],
            "version-1",
        )
        .unwrap();
        replace_for_passages(
            &conn,
            super::super::PassageKind::Document,
            "doc-1",
            Some("page:1"),
            &[passage("Budget: QAR 80,000.")],
            "version-2",
        )
        .unwrap();
        let current: (String, String) = conn
            .query_row(
                "SELECT value, document_version FROM source_facts WHERE ref_id='doc-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(current, ("QAR 80,000".into(), "version-2".into()));
        replace_for_passages(
            &conn,
            super::super::PassageKind::Document,
            "doc-1",
            Some("page:1"),
            &[],
            "version-3",
        )
        .unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM source_facts WHERE ref_id='doc-1'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn matching_obeys_meeting_document_and_exclusion_scope() {
        let conn = Connection::open_in_memory().unwrap();
        schema(&conn).unwrap();
        let add = |kind: &str, id: &str, value: &str| {
            conn.execute(
                "INSERT INTO source_facts
                 (kind, ref_id, title, subject, attribute, value, supporting_text,
                  document_version, verification_status, extraction_method, normalized_value)
                 VALUES (?1, ?2, ?2, 'atlas', 'budget', ?3, 'Budget: value', 'hash',
                         'source_exact', 'labeled_numeric_v1', ?3)",
                params![kind, id, value],
            )
            .unwrap();
        };
        add("transcript", "meeting-in", "100");
        add("transcript", "meeting-out", "200");
        add("document", "doc-in", "300");
        add("document", "doc-out", "400");
        let scope = super::super::SearchScope {
            meetings: super::super::MeetingScope::Ids(vec![
                "meeting-in".into(),
                "meeting-out".into(),
            ]),
            document_ids: vec!["doc-in".into()],
            excluded_meeting_ids: vec!["meeting-out".into()],
            ..Default::default()
        };
        let found = matching(&conn, &scope, "What is the budget?", 20).unwrap();
        let ids: Vec<_> = found.iter().map(|f| f.ref_id.as_str()).collect();
        assert_eq!(ids, vec!["meeting-in", "doc-in"]);
    }

    #[test]
    fn conflicts_require_the_same_subject_attribute_and_conditions() {
        let record = |subject: &str, value: &str, conditions: Option<&str>| FactRecord {
            kind: "document".into(),
            ref_id: value.into(),
            sub_ref: None,
            title: "Spec".into(),
            subject: subject.into(),
            attribute: "pressure".into(),
            value: value.into(),
            unit: Some("bar".into()),
            conditions: conditions.map(str::to_string),
            location: "page:1".into(),
            evidence: String::new(),
            document_version: "h".into(),
            verification_status: "source_exact".into(),
            extraction_method: "labeled_numeric_v1".into(),
            normalized_value: value.into(),
        };
        assert!(has_conflict(&[
            record("pump A", "20", None),
            record("pump a", "22", None)
        ]));
        assert!(!has_conflict(&[
            record("pump A", "20", Some("cold")),
            record("pump A", "22", Some("hot"))
        ]));
        assert!(!has_conflict(&[
            record("pump A", "20", None),
            record("pump B", "22", None)
        ]));
    }
}
