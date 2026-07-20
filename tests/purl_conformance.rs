//! Conformance test against the official Package URL specification suite.
//!
//! The suite files are vendored under `tests/purl-spec/`, fetched from
//! `package-url/purl-spec` at the commit pinned in
//! `scripts/update-purl-spec-tests.sh` and verified against
//! `scripts/purl-spec-tests.sha256`. Every case is run through the public API
//! and reconciled against `KNOWN_GAPS`, the list of cases the crate does not
//! yet satisfy.
//!
//! The single test is a two-sided guard:
//!
//!   * a non-conformant case that is *not* listed in `KNOWN_GAPS` fails the test
//!     as a regression;
//!   * a `KNOWN_GAPS` entry that no non-conformant case reproduces fails the test
//!     so that a gap fixed upstream (or by this crate) gets pruned.
//!
//! To bump the suite, edit the pin in the update script, run it with
//! `--refresh`, then re-baseline:
//! `PURL_CONFORMANCE_DUMP=1 cargo test --test purl_conformance -- --nocapture`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use packageurl::PackageUrl;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Vendored test-suite model (schema: schemas/purl-test.schema-0.1.json upstream)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct SuiteFile {
    #[serde(default)]
    tests: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    description: String,
    test_type: TestType,
    #[serde(default)]
    expected_failure: bool,
    input: Io,
    #[serde(default)]
    expected_output: Option<Io>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TestType {
    Parse,
    Build,
    Roundtrip,
}

impl TestType {
    fn as_str(self) -> &'static str {
        match self {
            TestType::Parse => "parse",
            TestType::Build => "build",
            TestType::Roundtrip => "roundtrip",
        }
    }
}

/// `input` and `expected_output` are a purl string for parse/roundtrip and a
/// component object for build; the two JSON shapes are disjoint, so `untagged`
/// resolves them unambiguously.
#[derive(Deserialize)]
#[serde(untagged)]
enum Io {
    Purl(String),
    Components(Box<Components>),
}

impl Io {
    fn purl(&self) -> Option<&str> {
        match self {
            Io::Purl(purl) => Some(purl.as_str()),
            Io::Components(_) => None,
        }
    }

    fn components(&self) -> Option<&Components> {
        match self {
            Io::Components(components) => Some(components),
            Io::Purl(_) => None,
        }
    }
}

#[derive(Deserialize)]
struct Components {
    #[serde(rename = "type", default)]
    ty: Option<String>,
    #[serde(default)]
    namespace: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    qualifiers: Option<BTreeMap<String, String>>,
    #[serde(default)]
    subpath: Option<String>,
}

// ---------------------------------------------------------------------------
// Known non-conformances
// ---------------------------------------------------------------------------

/// Cases the crate does not yet satisfy, as `(logical_key, note)` entries.
/// Rebuild from the `PURL_CONFORMANCE_DUMP` block after refreshing the suite.
/// Empty: every case at the pinned suite commit is conformant.
#[rustfmt::skip]
static KNOWN_GAPS: &[(&str, &str)] = &[];

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

/// A conformance verdict: `Ok(())` is conformant, `Err(detail)` is not.
type Check = Result<(), String>;

fn evaluate(case: &Case) -> Check {
    match case.test_type {
        TestType::Parse => eval_parse(case),
        TestType::Build => eval_build(case),
        TestType::Roundtrip => eval_roundtrip(case),
    }
}

fn eval_parse(case: &Case) -> Check {
    let input = case.input.purl().expect("parse input is a purl string");
    let parsed = PackageUrl::from_str(input);
    if case.expected_failure {
        return match parsed {
            Err(_) => Ok(()),
            Ok(purl) => Err(format!("expected parse to fail, got {purl}")),
        };
    }
    let expected = case
        .expected_output
        .as_ref()
        .and_then(Io::components)
        .expect("parse output is a component object");
    match parsed {
        Err(err) => Err(format!("expected success, parse failed: {err}")),
        Ok(purl) => compare(&purl, expected),
    }
}

fn eval_roundtrip(case: &Case) -> Check {
    let input = case.input.purl().expect("roundtrip input is a purl string");
    let parsed = PackageUrl::from_str(input);
    if case.expected_failure {
        return match parsed {
            Err(_) => Ok(()),
            Ok(purl) => Err(format!("expected roundtrip to fail, got {purl}")),
        };
    }
    let expected = case
        .expected_output
        .as_ref()
        .and_then(Io::purl)
        .expect("roundtrip output is a purl string");
    match parsed {
        Err(err) => Err(format!("expected success, parse failed: {err}")),
        Ok(purl) => {
            let got = purl.to_string();
            if got == expected {
                Ok(())
            } else {
                Err(format!("got {got:?}, want {expected:?}"))
            }
        }
    }
}

fn eval_build(case: &Case) -> Check {
    let input = case.input.components().expect("build input is components");
    let built = build(input);
    if case.expected_failure {
        return match built {
            Err(_) => Ok(()),
            Ok(got) => Err(format!("expected build to fail, got {got:?}")),
        };
    }
    let expected = case
        .expected_output
        .as_ref()
        .and_then(Io::purl)
        .expect("build output is a purl string");
    match built {
        Err(err) => Err(format!("expected success, build failed: {err}")),
        Ok(got) if got == expected => Ok(()),
        Ok(got) => Err(format!("got {got:?}, want {expected:?}")),
    }
}

/// Build a canonical purl from decoded components via the public builder API,
/// validated like the parser validates. A null type or name maps to `""`,
/// which the builder rejects.
fn build(components: &Components) -> Result<String, packageurl::Error> {
    let mut purl = PackageUrl::new(
        components.ty.as_deref().unwrap_or(""),
        components.name.as_deref().unwrap_or(""),
    )?;
    if let Some(namespace) = components.namespace.as_deref() {
        purl.with_namespace(namespace)?;
    }
    if let Some(version) = components.version.as_deref() {
        purl.with_version(version)?;
    }
    if let Some(subpath) = components.subpath.as_deref() {
        purl.with_subpath(subpath)?;
    }
    for (key, value) in components.qualifiers.iter().flatten() {
        purl.add_qualifier(key.as_str(), value.as_str())?;
    }
    purl.validate()?;
    Ok(purl.to_string())
}

fn compare(purl: &PackageUrl, expected: &Components) -> Check {
    let mut diffs: Vec<String> = Vec::new();

    let want_ty = expected.ty.as_deref().unwrap_or("");
    if purl.ty() != want_ty {
        diffs.push(format!("type got {:?} want {want_ty:?}", purl.ty()));
    }
    let want_name = expected.name.as_deref().unwrap_or("");
    if purl.name() != want_name {
        diffs.push(format!("name got {:?} want {want_name:?}", purl.name()));
    }
    if purl.namespace() != expected.namespace.as_deref() {
        diffs.push(format!(
            "namespace got {:?} want {:?}",
            purl.namespace(),
            expected.namespace.as_deref()
        ));
    }
    if purl.version() != expected.version.as_deref() {
        diffs.push(format!(
            "version got {:?} want {:?}",
            purl.version(),
            expected.version.as_deref()
        ));
    }
    if purl.subpath() != expected.subpath.as_deref() {
        diffs.push(format!(
            "subpath got {:?} want {:?}",
            purl.subpath(),
            expected.subpath.as_deref()
        ));
    }
    let want_quals: BTreeMap<&str, &str> = expected
        .qualifiers
        .iter()
        .flatten()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect();
    let got_quals: BTreeMap<&str, &str> = purl
        .qualifiers()
        .iter()
        .map(|(key, value)| (key.as_ref(), value.as_ref()))
        .collect();
    if want_quals != got_quals {
        diffs.push(format!("qualifiers got {got_quals:?} want {want_quals:?}"));
    }

    if diffs.is_empty() {
        Ok(())
    } else {
        Err(diffs.join(", "))
    }
}

// ---------------------------------------------------------------------------
// Case identity
// ---------------------------------------------------------------------------

fn opt(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("")
}

/// A stable identity for a case: `(file, test_type, input)`. Cases that share
/// this key always share their expected outcome, so collapsing them is safe.
fn logical_key(file: &str, case: &Case) -> String {
    let test_type = case.test_type.as_str();
    match &case.input {
        Io::Purl(purl) => format!("{file}::{test_type}::purl={purl}"),
        Io::Components(components) => {
            let qualifiers = components
                .qualifiers
                .as_ref()
                .map(|map| {
                    map.iter()
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect::<Vec<_>>()
                        .join(",")
                })
                .unwrap_or_default();
            format!(
                "{file}::{test_type}::build[type={}|ns={}|name={}|ver={}|qual={}|sub={}]",
                opt(&components.ty),
                opt(&components.namespace),
                opt(&components.name),
                opt(&components.version),
                qualifiers,
                opt(&components.subpath),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Loads every case from the vendored suite.
fn load() -> Vec<(String, Case)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/purl-spec");
    assert!(
        root.join("spec").is_dir(),
        "vendored purl-spec suite missing at {} — run scripts/update-purl-spec-tests.sh",
        root.display()
    );
    let mut cases = Vec::new();
    for sub in ["spec", "types"] {
        let dir = root.join(sub);
        let mut files: Vec<PathBuf> = fs::read_dir(&dir)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()))
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
            .collect();
        files.sort();
        for file in files {
            let name = file
                .file_name()
                .expect("directory entry has a file name")
                .to_string_lossy()
                .into_owned();
            let text = fs::read_to_string(&file)
                .unwrap_or_else(|err| panic!("cannot read {}: {err}", file.display()));
            let suite: SuiteFile = serde_json::from_str(&text)
                .unwrap_or_else(|err| panic!("cannot parse {name}: {err}"));
            for case in suite.tests {
                cases.push((name.clone(), case));
            }
        }
    }
    cases
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

/// Print every non-conformant key so `KNOWN_GAPS` can be rebuilt. Enabled with
/// `PURL_CONFORMANCE_DUMP=1`.
fn dump(cases: &[(String, Case)], gaps: &HashMap<&str, &str>) {
    let mut seen: HashSet<String> = HashSet::new();
    let mut lines: Vec<String> = Vec::new();
    for (file, case) in cases {
        if let Err(detail) = evaluate(case) {
            let key = logical_key(file, case);
            if seen.insert(key.clone()) {
                let status = if gaps.contains_key(key.as_str()) {
                    "GAP"
                } else {
                    "NEW"
                };
                lines.push(format!("{status}\t{key}\t{detail}"));
            }
        }
    }
    lines.sort();
    println!("=== PURL_CONFORMANCE_DUMP BEGIN ({}) ===", lines.len());
    for line in &lines {
        println!("{line}");
    }
    println!("=== PURL_CONFORMANCE_DUMP END ===");
}

#[test]
fn purl_spec_conformance() {
    let cases = load();
    assert!(!cases.is_empty(), "the vendored suite contains no cases");

    let mut gaps: HashMap<&str, &str> = HashMap::new();
    for &(key, note) in KNOWN_GAPS {
        assert!(
            gaps.insert(key, note).is_none(),
            "duplicate KNOWN_GAPS key: {key}"
        );
    }

    let mut total = 0usize;
    let mut nonconformant: HashSet<String> = HashSet::new();
    let mut regressions: Vec<String> = Vec::new();

    for (file, case) in &cases {
        total += 1;
        if let Err(detail) = evaluate(case) {
            let key = logical_key(file, case);
            if nonconformant.insert(key.clone()) && !gaps.contains_key(key.as_str()) {
                regressions.push(format!("  {key}\n      {} — {detail}", case.description));
            }
        }
    }

    let mut resolved: Vec<String> = KNOWN_GAPS
        .iter()
        .filter(|(key, _)| !nonconformant.contains(*key))
        .map(|(key, note)| format!("  {key} — {note}"))
        .collect();

    if std::env::var_os("PURL_CONFORMANCE_DUMP").is_some() {
        dump(&cases, &gaps);
    }

    regressions.sort();
    resolved.sort();

    if regressions.is_empty() && resolved.is_empty() {
        return;
    }

    let mut report = vec![format!(
        "purl-spec conformance: {total} cases, {} non-conformant, {} known-gap keys",
        nonconformant.len(),
        KNOWN_GAPS.len(),
    )];
    if !regressions.is_empty() {
        report.push(format!(
            "\nREGRESSIONS — {} case(s) fail that are not in KNOWN_GAPS:",
            regressions.len()
        ));
        report.extend(regressions);
    }
    if !resolved.is_empty() {
        report.push(format!(
            "\nRESOLVED — {} KNOWN_GAPS entr(y|ies) now conformant; remove them:",
            resolved.len()
        ));
        report.extend(resolved);
    }
    panic!("{}", report.join("\n"));
}
