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

#[derive(Clone, Copy)]
enum Category {
    /// Qualifier values are canonicalized without encoding `/` or `,`.
    QualifierEncoding,
    /// A type's name/version normalization rule is not fully applied.
    Normalization,
    /// A type-specific validity rule is not enforced when parsing.
    ParseAcceptsInvalid,
    /// A valid purl is rejected when parsing.
    ParseRejectsValid,
    /// The builder accepts components the spec rejects.
    BuildAcceptsInvalid,
}

impl Category {
    fn tag(self) -> &'static str {
        match self {
            Category::QualifierEncoding => "qualifier-encoding",
            Category::Normalization => "normalization",
            Category::ParseAcceptsInvalid => "parse-accepts-invalid",
            Category::ParseRejectsValid => "parse-rejects-valid",
            Category::BuildAcceptsInvalid => "build-accepts-invalid",
        }
    }
}

struct KnownGap {
    key: &'static str,
    category: Category,
    note: &'static str,
}

/// Cases the crate does not yet satisfy, keyed by `logical_key` and grouped by
/// cause. Rebuild from the `PURL_CONFORMANCE_DUMP` block after refreshing the suite.
/// Total: 77 (QualifierEncoding: 45, Normalization: 5, ParseAcceptsInvalid: 12, ParseRejectsValid: 2, BuildAcceptsInvalid: 13).
#[rustfmt::skip]
static KNOWN_GAPS: &[KnownGap] = &[
    // ── Qualifier value encoding: '/' and ',' are not percent-encoded (issue category 1). ──
    KnownGap { key: "bazel-test.json::build::build[type=bazel|ns=|name=rules_java|ver=7.8.0|qual=repository_url=https://example.org/bazel-registry|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "bazel-test.json::roundtrip::purl=pkg:bazel/rules_java@7.8.0?repository_url=https://bcr.bazel.build/", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "bazel-test.json::roundtrip::purl=pkg:bazel/rules_java@7.8.0?repository_url=https://example.org/bazel-registry/", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "generic-test.json::build::build[type=generic|ns=|name=bitwarderl|ver=|qual=vcs_url=git+https://git.fsfe.org/dxtr/bitwarderl@cc55108da32|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "generic-test.json::build::build[type=generic|ns=|name=openssl|ver=1.1.10g|qual=checksum=sha256:de4d501267da,download_url=https://openssl.org/source/openssl-1.1.0g.tar.gz|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "generic-test.json::roundtrip::purl=pkg:generic/bitwarderl?vcs_url=git%2Bhttps://git.fsfe.org/dxtr/bitwarderl%40cc55108da32", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "generic-test.json::roundtrip::purl=pkg:generic/openssl@1.1.10g?download_url=https://openssl.org/source/openssl-1.1.0g.tar.gz&checksum=sha256:de4d501267da", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "hex-test.json::build::build[type=hex|ns=|name=bar|ver=1.2.3|qual=repository_url=https://myrepo.example.com|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "hex-test.json::roundtrip::purl=pkg:hex/bar@1.2.3?repository_url=https://myrepo.example.com", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "huggingface-test.json::build::build[type=huggingface|ns=microsoft|name=deberta-v3-base|ver=559062ad13d311b87b2c455e67dcd5f1c8f65111|qual=repository_url=https://hub-ci.huggingface.co|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "huggingface-test.json::roundtrip::purl=pkg:huggingface/microsoft/deberta-v3-base@559062ad13d311b87b2c455e67dcd5f1c8f65111?repository_url=https://hub-ci.huggingface.co", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "julia-test.json::build::build[type=julia|ns=|name=RegisterQD|ver=0.3.1|qual=repository_url=https://github.com/HolyLab/HolyLabRegistry,uuid=ac24ea0c-1830-11e9-18d4-81f172323054|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "julia-test.json::roundtrip::purl=pkg:julia/RegisterQD@0.3.1?repository_url=https://github.com/HolyLab/HolyLabRegistry&uuid=ac24ea0c-1830-11e9-18d4-81f172323054", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "luarocks-test.json::build::build[type=luarocks|ns=username|name=packagename|ver=0.1.0-1|qual=repository_url=https://example.com/private_rocks_server/|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "luarocks-test.json::roundtrip::purl=pkg:luarocks/username/packagename@0.1.0-1?repository_url=https://example.com/private_rocks_server/", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "maven-test.json::build::build[type=maven|ns=groovy|name=groovy|ver=1.0|qual=repository_url=https://maven.google.com|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "maven-test.json::build::build[type=maven|ns=org.apache.xmlgraphics|name=batik-anim|ver=1.9.1|qual=classifier=foo,repository_url=repo.spring.io/release|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "maven-test.json::build::build[type=maven|ns=org.apache.xmlgraphics|name=batik-anim|ver=1.9.1|qual=classifier=sources,repository_url=repo.spring.io/release|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "maven-test.json::roundtrip::purl=pkg:Maven/org.apache.xmlgraphics/batik-anim@1.9.1?classifier=sources&repositorY_url=https://repo.spring.io/release", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "maven-test.json::roundtrip::purl=pkg:Maven/org.apache.xmlgraphics/batik-anim@1.9.1?type=pom&repositorY_url=repo.spring.io/release", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "maven-test.json::roundtrip::purl=pkg:maven/groovy/groovy@1.0?repository_url=https://maven.google.com", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "maven-test.json::roundtrip::purl=pkg:maven/org.apache.xmlgraphics/batik-anim@1.9.1?classifier=sources&repository_url=repo.spring.io/release", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "maven-test.json::roundtrip::purl=pkg:maven/org.apache.xmlgraphics/batik-anim@1.9.1?type=war&repository_url=https://repo.spring.io/release", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::build::build[type=mlflow|ns=|name=CreditFraud|ver=3|qual=repository_url=https://westus2.api.azureml.ms/mlflow/v1.0/subscriptions/a50f2011-fab8-4164-af23-c62881ef8c95/resourceGroups/TestResourceGroup/providers/Microsoft.MachineLearningServices/workspaces/TestWorkspace|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::build::build[type=mlflow|ns=|name=creditfraud|ver=3|qual=repository_url=https://adb-5245952564735461.0.azuredatabricks.net/api/2.0/mlflow|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::build::build[type=mlflow|ns=|name=creditfraud|ver=3|qual=repository_url=https://westus2.api.azureml.ms/mlflow/v1.0/subscriptions/a50f2011-fab8-4164-af23-c62881ef8c95/resourceGroups/TestResourceGroup/providers/Microsoft.MachineLearningServices/workspaces/TestWorkspace|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::build::build[type=mlflow|ns=|name=trafficsigns|ver=10|qual=model_uuid=36233173b22f4c89b451f1228d700d49,repository_url=https://adb-5245952564735461.0.azuredatabricks.net/api/2.0/mlflow,run_id=410a3121-2709-4f88-98dd-dba0ef056b0a|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::roundtrip::purl=pkg:mlflow/CreditFraud@3?repository_url=https://adb-5245952564735461.0.azuredatabricks.net/api/2.0/mlflow", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::roundtrip::purl=pkg:mlflow/CreditFraud@3?repository_url=https://westus2.api.azureml.ms/mlflow/v1.0/subscriptions/a50f2011-fab8-4164-af23-c62881ef8c95/resourceGroups/TestResourceGroup/providers/Microsoft.MachineLearningServices/workspaces/TestWorkspace", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::roundtrip::purl=pkg:mlflow/creditfraud@3?repository_url=https://adb-5245952564735461.0.azuredatabricks.net/api/2.0/mlflow", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::roundtrip::purl=pkg:mlflow/creditfraud@3?repository_url=https://westus2.api.azureml.ms/mlflow/v1.0/subscriptions/a50f2011-fab8-4164-af23-c62881ef8c95/resourceGroups/TestResourceGroup/providers/Microsoft.MachineLearningServices/workspaces/TestWorkspace", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::roundtrip::purl=pkg:mlflow/trafficsigns@10?model_uuid=36233173b22f4c89b451f1228d700d49&repository_url=https://adb-5245952564735461.0.azuredatabricks.net/api/2.0/mlflow&run_id=410a3121-2709-4f88-98dd-dba0ef056b0a", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "mlflow-test.json::roundtrip::purl=pkg:mlflow/trafficsigns@10?model_uuid=36233173b22f4c89b451f1228d700d49&run_id=410a3121-2709-4f88-98dd-dba0ef056b0a&repository_url=https://adb-5245952564735461.0.azuredatabricks.net/api/2.0/mlflow", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "npm-test.json::build::build[type=npm|ns=|name=mypackage|ver=12.4.5|qual=vcs_url=git://host.com/path/to/repo.git@4345abcd34343|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "npm-test.json::roundtrip::purl=pkg:npm/mypackage@12.4.5?vcs_url=git://host.com/path/to/repo.git%404345abcd34343", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "oci-test.json::build::build[type=oci|ns=|name=debian|ver=sha256:244fd47e07d10|qual=arch=amd64,repository_url=docker.io/library/debian,tag=latest|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "oci-test.json::build::build[type=oci|ns=|name=debian|ver=sha256:244fd47e07d10|qual=repository_url=ghcr.io/debian,tag=bullseye|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "oci-test.json::build::build[type=oci|ns=|name=static|ver=sha256:244fd47e07d10|qual=repository_url=gcr.io/distroless/static,tag=latest|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "oci-test.json::roundtrip::purl=pkg:oci/debian@sha256%3A244fd47e07d10?repository_url=docker.io/library/debian&arch=amd64&tag=latest", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "oci-test.json::roundtrip::purl=pkg:oci/debian@sha256%3A244fd47e07d10?repository_url=ghcr.io/debian&tag=bullseye", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "oci-test.json::roundtrip::purl=pkg:oci/static@sha256%3A244fd47e07d10?repository_url=gcr.io/distroless/static&tag=latest", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "otp-test.json::build::build[type=otp|ns=|name=asn1|ver=5.4.1|qual=arch=amd64,platform=linux,repository_url=https://github.com/erlang/otp,vcs_url=git+https://github.com/erlang/otp.git|sub=src/asn1ct.erl]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "otp-test.json::roundtrip::purl=pkg:otp/asn1@5.4.1?arch=amd64&platform=linux&repository_url=https://github.com/erlang/otp&vcs_url=git%2Bhttps://github.com/erlang/otp.git#src/asn1ct.erl", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "specification-test.json::build::build[type=generic|ns=|name=openssl|ver=1.1.10g|qual=checksum=sha1:ad9503c3e994a4f,sha256:41bf9088b3a1e6c1ef1d|sub=]", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },
    KnownGap { key: "specification-test.json::roundtrip::purl=pkg:generic/bitwarderl?checksum=sha1:ad9503c3e994a4f%2Csha256:41bf9088b3a1e6c1ef1d", category: Category::QualifierEncoding, note: "qualifier value '/' and ',' left literal (ENCODE_SET omits them)" },

    // ── Type normalization: name/version case rules not applied (issue category 2; also huggingface, mlflow). ──
    KnownGap { key: "composer-test.json::parse::purl=pkg:composer/Laravel/Laravel@5.5.0", category: Category::Normalization, note: "type-specific name/version normalization not applied" },
    KnownGap { key: "composer-test.json::roundtrip::purl=pkg:composer/Laravel/Laravel@5.5.0", category: Category::Normalization, note: "type-specific name/version normalization not applied" },
    KnownGap { key: "huggingface-test.json::parse::purl=pkg:huggingface/EleutherAI/gpt-neo-1.3B@797174552AE47F449AB70B684CABCB6603E5E85E", category: Category::Normalization, note: "type-specific name/version normalization not applied" },
    KnownGap { key: "huggingface-test.json::roundtrip::purl=pkg:huggingface/EleutherAI/gpt-neo-1.3B@797174552AE47F449AB70B684CABCB6603E5E85E", category: Category::Normalization, note: "type-specific name/version normalization not applied" },
    KnownGap { key: "mlflow-test.json::parse::purl=pkg:mlflow/CreditFraud@3?repository_url=https://adb-5245952564735461.0.azuredatabricks.net/api/2.0/mlflow", category: Category::Normalization, note: "type-specific name/version normalization not applied" },

    // ── Required/typed components: per-type validity not enforced on parse (issue category 4). ──
    KnownGap { key: "chrome-extension-test.json::parse::purl=pkg:chrome-extension/44444algnefjeiefhmpklpfiohadpglk", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "chrome-extension-test.json::parse::purl=pkg:chrome-extension/dlpngalgnefjeiefhmpklpfiohadpglk@1.2.3-beta", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "chrome-extension-test.json::parse::purl=pkg:chrome-extension/dlpngalgnefjeiefhmpklpfiohadpglk@1.2.3.4.5", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "chrome-extension-test.json::parse::purl=pkg:chrome-extension/dogs", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "cpan-test.json::parse::purl=pkg:cpan/GDT/URI::PackageURL", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "cpan-test.json::parse::purl=pkg:cpan/LWP::UserAgent@6.7.6", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "julia-test.json::parse::purl=pkg:julia/Dates", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "otp-test.json::parse::purl=pkg:otp/namespace/hex@2.1.1", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "swift-test.json::parse::purl=pkg:swift/Alamofire@5.4.3", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "swift-test.json::parse::purl=pkg:swift/github.com/Alamofire/@5.4.3", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "vcpkg-test.json::parse::purl=pkg:vcpkg/boost/asio@1.84.0", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },
    KnownGap { key: "vscode-extension-test.json::parse::purl=pkg:vscode-extension/java@1.46.2025091308", category: Category::ParseAcceptsInvalid, note: "type-specific validity rule not enforced on parse" },

    // ── Scoped parsing: an unencoded '@' scope with a subpath is rejected (issue category 3). ──
    KnownGap { key: "npm-test.json::parse::purl=pkg:npm/@babel/core#/googleapis/api/annotations/", category: Category::ParseRejectsValid, note: "valid purl rejected: unencoded '@' scope with a subpath" },
    KnownGap { key: "npm-test.json::roundtrip::purl=pkg:npm/@babel/core#/googleapis/api/annotations/", category: Category::ParseRejectsValid, note: "valid purl rejected: unencoded '@' scope with a subpath" },

    // ── Builder validation: the builder accepts components the spec rejects. ──
    KnownGap { key: "cpan-test.json::build::build[type=cpan|ns=GDT|name=URI::PackageURL|ver=|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "cpan-test.json::build::build[type=cpan|ns=|name=Perl-Version|ver=1.013|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "cran-test.json::build::build[type=cran|ns=|name=|ver=0.9.1|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "hackage-test.json::build::build[type=hackage|ns=|name=|ver=|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "julia-test.json::build::build[type=julia|ns=|name=|ver=1.9.0|qual=uuid=ade2ca70-3891-5945-98fb-dc099432e06a|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "opam-test.json::build::build[type=opam|ns=|name=|ver=|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "otp-test.json::build::build[type=otp|ns=namespace|name=hex|ver=2.1.1|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "specification-test.json::build::build[type=maven|ns=|name=|ver=|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "swift-test.json::build::build[type=swift|ns=github.com/Alamofire|name=|ver=5.4.3|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "swift-test.json::build::build[type=swift|ns=|name=Alamofire|ver=5.4.3|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "vcpkg-test.json::build::build[type=vcpkg|ns=boost|name=asio|ver=1.84.0|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "vcpkg-test.json::build::build[type=vcpkg|ns=|name=|ver=1.0.8|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
    KnownGap { key: "vscode-extension-test.json::build::build[type=vscode-extension|ns=|name=java|ver=1.46.2025091308|qual=|sub=]", category: Category::BuildAcceptsInvalid, note: "builder accepts components the spec rejects" },
];

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

/// Build a canonical purl from decoded components via the public builder API.
/// A null type or name maps to `""` (the builder rejects an empty type; an empty
/// name currently builds, which surfaces as a tracked gap).
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
fn dump(cases: &[(String, Case)], gaps: &HashMap<&str, &KnownGap>) {
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

    let mut gaps: HashMap<&str, &KnownGap> = HashMap::new();
    for gap in KNOWN_GAPS {
        assert!(
            gaps.insert(gap.key, gap).is_none(),
            "duplicate KNOWN_GAPS key: {}",
            gap.key
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
        .filter(|gap| !nonconformant.contains(gap.key))
        .map(|gap| format!("  {}  [{}] — {}", gap.key, gap.category.tag(), gap.note))
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
