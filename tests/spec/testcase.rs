use std::borrow::Cow;
use std::collections::HashMap;
use serde::Deserialize;


#[derive(Deserialize)]
#[serde(untagged)]
pub enum PurlOrString<'a> {
    String(Cow<'a, str>),
    PurlComponent(PurlComponents<'a>),
}


#[derive(Deserialize)]
#[allow(dead_code)]
pub struct PurlComponents<'a> {
    #[serde(rename = "type")]
    pub ty: Option<Cow<'a, str>>,
    pub namespace: Option<Cow<'a, str>>,
    pub name: Option<Cow<'a, str>>,
    pub version: Option<Cow<'a, str>>,
    pub qualifiers: Option<HashMap<Cow<'a, str>, Cow<'a, str>>>,
    pub subpath: Option<Cow<'a, str>>,
}


#[derive(Deserialize)]
#[allow(dead_code)]
pub struct SpecTestCase<'a> {
    pub description: Cow<'a, str>,
    pub test_group: Cow<'a, str>,
    pub test_type: Cow<'a, str>,
    pub input: PurlOrString<'a>,
    pub expected_output: Option<PurlOrString<'a>>,
    pub expected_failure: bool,
    pub expected_failure_reason: Option<Cow<'a, str>>,
}


#[derive(Deserialize)]
pub struct TestSuite<'a> {
    pub tests: Vec<SpecTestCase<'a>>,
}
