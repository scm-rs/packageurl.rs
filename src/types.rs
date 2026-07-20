//! Per-type rules, transcribed from the purl-spec type definitions
//! (`types/*-definition.json` in the suite pinned by
//! `scripts/update-purl-spec-tests.sh`).

use std::borrow::Cow;

use super::errors::{Error, Result};
use super::purl::PackageUrl;

pub(crate) enum Namespace {
    Optional,
    Required,
    Prohibited,
}

pub(crate) struct TypeRules {
    ty: &'static str,
    pub(crate) namespace: Namespace,
    pub(crate) lowercase_name: bool,
    pub(crate) lowercase_namespace: bool,
    pub(crate) lowercase_version: bool,
    pub(crate) required_qualifiers: &'static [&'static str],
}

impl TypeRules {
    const DEFAULT: TypeRules = TypeRules {
        ty: "",
        namespace: Namespace::Optional,
        lowercase_name: false,
        lowercase_namespace: false,
        lowercase_version: false,
        required_qualifiers: &[],
    };
}

#[rustfmt::skip]
static RULES: &[TypeRules] = &[
    TypeRules { ty: "apk", lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "bitbucket", lowercase_name: true, lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "bitnami", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "cargo", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "chrome-extension", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "cocoapods", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "composer", namespace: Namespace::Required, lowercase_name: true, lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "conda", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "cpan", namespace: Namespace::Required, ..TypeRules::DEFAULT },
    TypeRules { ty: "cran", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "deb", lowercase_name: true, lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "gem", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "github", lowercase_name: true, lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "golang", lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "hackage", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "hex", lowercase_name: true, lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "huggingface", namespace: Namespace::Required, lowercase_version: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "julia", namespace: Namespace::Prohibited, required_qualifiers: &["uuid"], ..TypeRules::DEFAULT },
    TypeRules { ty: "mlflow", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "npm", lowercase_name: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "nuget", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "oci", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "otp", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "pub", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "pypi", namespace: Namespace::Prohibited, lowercase_name: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "qpkg", lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "rpm", lowercase_namespace: true, ..TypeRules::DEFAULT },
    TypeRules { ty: "swift", namespace: Namespace::Required, ..TypeRules::DEFAULT },
    TypeRules { ty: "vcpkg", namespace: Namespace::Prohibited, ..TypeRules::DEFAULT },
    TypeRules { ty: "vscode-extension", namespace: Namespace::Required, ..TypeRules::DEFAULT },
];

pub(crate) fn rules(ty: &str) -> Option<&'static TypeRules> {
    RULES.iter().find(|rules| rules.ty == ty)
}

/// Applies the canonicalization that depends on other components: on
/// Databricks servers the mlflow name is case insensitive and lowercased,
/// while other servers keep it as-is (mlflow-definition.json).
pub(crate) fn canonicalize(purl: &mut PackageUrl<'_>) {
    if purl.ty == "mlflow"
        && purl
            .qualifiers
            .get("repository_url")
            .is_some_and(|url| url.contains("databricks"))
        && purl.name.chars().any(|c| c.is_uppercase())
    {
        purl.name = Cow::Owned(purl.name.to_lowercase());
    }
}

/// Validates the whole purl against its type's rules; unknown types have none.
pub(crate) fn validate(purl: &PackageUrl<'_>) -> Result<()> {
    let Some(rules) = rules(&purl.ty) else {
        return Ok(());
    };
    match rules.namespace {
        Namespace::Required if purl.namespace.is_none() => {
            return Err(Error::TypeRequiresNamespace(purl.ty.to_string()));
        }
        Namespace::Prohibited if purl.namespace.is_some() => {
            return Err(Error::TypeProhibitsNamespace(purl.ty.to_string()));
        }
        _ => {}
    }
    for key in rules.required_qualifiers {
        if !purl.qualifiers.contains_key(*key) {
            return Err(Error::MissingRequiredQualifier(
                purl.ty.to_string(),
                (*key).to_string(),
            ));
        }
    }
    match purl.ty.as_ref() {
        // The name is a 32-character extension id and the version has 1-4
        // dotted integer segments (chrome-extension-definition.json).
        "chrome-extension" => {
            if purl.name.len() != 32 || !purl.name.bytes().all(|b| b.is_ascii_lowercase()) {
                return Err(Error::InvalidName(
                    purl.ty.to_string(),
                    purl.name.to_string(),
                ));
            }
            if let Some(version) = purl.version.as_deref() {
                let valid = version.split('.').count() <= 4
                    && version.split('.').all(|segment| {
                        !segment.is_empty() && segment.bytes().all(|b| b.is_ascii_digit())
                    });
                if !valid {
                    return Err(Error::InvalidVersion(
                        purl.ty.to_string(),
                        version.to_string(),
                    ));
                }
            }
        }
        // The name is a distribution name and must not contain "::"
        // (cpan-definition.json).
        "cpan" if purl.name.contains("::") => {
            return Err(Error::InvalidName(
                purl.ty.to_string(),
                purl.name.to_string(),
            ));
        }
        _ => {}
    }
    Ok(())
}
