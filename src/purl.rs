use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::str::FromStr;

use percent_encoding::AsciiSet;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::errors::Error;
use super::errors::Result;
use super::parser;
use super::utils::{to_lowercase, PercentCodec};
use super::validation;

const ENCODE_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'`')
    .add(b'?')
    .add(b'{')
    .add(b'}')
    .add(b'/')
    // .add(b':')
    .add(b';')
    .add(b'=')
    .add(b'+')
    .add(b'@')
    .add(b'\\')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'|');

/// A Package URL.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PackageUrl<'a> {
    /// The package URL type.
    pub(crate) ty: Cow<'a, str>,
    /// The optional namespace
    pub(crate) namespace: Option<Cow<'a, str>>,
    /// The package name.
    pub(crate) name: Cow<'a, str>,
    /// The optional package version.
    pub(crate) version: Option<Cow<'a, str>>,
    /// The package qualifiers.
    pub(crate) qualifiers: HashMap<Cow<'a, str>, Cow<'a, str>>,
    /// The package subpath.
    pub(crate) subpath: Option<Cow<'a, str>>,
}

impl<'a> PackageUrl<'a> {
    /// Create a new Package URL with the provided type and name.
    ///
    /// # Type
    /// The Package URL type must be valid, otherwise an error will be returned.
    /// The type can only be composed of ASCII letters and numbers, '.', '+'
    /// and '-' (period, plus and dash). It cannot start with a number and
    /// cannot contain spaces.
    ///
    /// # Name
    /// The package name will be canonicalize depending on the type: for instance,
    /// 'bitbucket' packages have a case-insensitive name, so the name will be
    /// lowercased if needed.
    ///
    /// # Example
    /// ```rust
    /// # extern crate packageurl;
    /// assert!( packageurl::PackageUrl::new("cargo", "packageurl").is_ok() );
    /// assert!( packageurl::PackageUrl::new("bad type", "packageurl").is_err() );
    /// ```
    pub fn new<T, N>(ty: T, name: N) -> Result<Self>
    where
        T: Into<Cow<'a, str>>,
        N: Into<Cow<'a, str>>,
    {
        let mut t = ty.into();
        let mut n = name.into();
        if validation::is_type_valid(&t) {
            t = to_lowercase(t);
            // lowercase name if required by type and needed
            match t.as_ref() {
                "bitbucket" | "deb" | "github" | "hex" | "npm" | "composer" | "mlflow" => {
                    n = to_lowercase(n);
                }
                "pypi" => {
                    n = to_lowercase(n);
                    if n.chars().any(|c| c == '_') {
                        n = Cow::Owned(n.replace('_', "-"));
                    }
                }
                _ => {}
            }

            Ok(Self::new_unchecked(t, n))
        } else {
            Err(Error::InvalidType(t.to_string()))
        }
    }

    /// Create a new Package URL without checking the type.
    fn new_unchecked<T, N>(ty: T, name: N) -> Self
    where
        T: Into<Cow<'a, str>>,
        N: Into<Cow<'a, str>>,
    {
        Self {
            ty: ty.into(),
            namespace: None,
            name: name.into(),
            version: None,
            qualifiers: HashMap::new(),
            subpath: None,
        }
    }

    /// Get the Package URL type.
    pub fn ty(&self) -> &str {
        self.ty.as_ref()
    }

    /// Get the optional namespace.
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_ref().map(Cow::as_ref)
    }

    /// Get the package name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the optional package version.
    pub fn version(&self) -> Option<&str> {
        self.version.as_ref().map(Cow::as_ref)
    }

    /// Get the package qualifiers
    pub fn qualifiers(&self) -> &HashMap<Cow<'a, str>, Cow<'a, str>> {
        &self.qualifiers
    }

    /// Get the optional package subpath.
    pub fn subpath(&self) -> Option<&str> {
        self.subpath.as_ref().map(Cow::as_ref)
    }

    /// Assign a namespace to the package.
    pub fn with_namespace<N>(&mut self, namespace: N) -> &mut Self
    where
        N: Into<Cow<'a, str>>,
    {
        let mut n = namespace.into();
        match self.ty.as_ref() {
            "bitbucket" | "deb" | "github" | "golang" | "hex" | "rpm" | "composer" => {
                n = to_lowercase(n);
            }
            _ => {}
        }

        self.namespace = Some(n);
        self
    }

    /// Clear the namespace
    pub fn without_namespace(&mut self) -> &mut Self {
        self.namespace = None;
        self
    }

    /// Assign a version to the package.
    pub fn with_version<V>(&mut self, version: V) -> &mut Self
    where
        V: Into<Cow<'a, str>>,
    {
        let mut v = version.into();
        match self.ty.as_ref() {
            "huggingface" => {
                v = to_lowercase(v);
            }
            _ => {}
        }
        self.version = Some(v);
        self
    }

    /// Clear the version
    pub fn without_version(&mut self) -> &mut Self {
        self.version = None;
        self
    }

    /// Assign a subpath to the package.
    ///
    /// Subpaths must not contain empty, local ('.') or parent ('..') segments,
    /// otherwise an error will be returned.
    pub fn with_subpath<S>(&mut self, subpath: S) -> Result<&mut Self>
    where
        S: Into<Cow<'a, str>>,
    {
        let s = subpath.into();
        for component in s.split('/') {
            if !validation::is_subpath_segment_valid(component) {
                return Err(Error::InvalidSubpathSegment(component.into()));
            }
        }
        self.subpath = Some(s);
        Ok(self)
    }

    /// Clear the subpath
    pub fn without_subpath(&mut self) -> &mut Self {
        self.subpath = None;
        self
    }

    /// Clear qualifiers
    pub fn clear_qualifiers(&mut self) -> &mut Self {
        self.qualifiers.clear();
        self
    }

    /// Add a qualifier to the package.
    pub fn add_qualifier<K, V>(&mut self, key: K, value: V) -> Result<&mut Self>
    where
        K: Into<Cow<'a, str>>,
        V: Into<Cow<'a, str>>,
    {
        let mut k = key.into();
        if !validation::is_qualifier_key_valid(&k) {
            Err(Error::InvalidKey(k.into()))
        } else {
            k = to_lowercase(k);
            self.qualifiers.insert(k, value.into());
            Ok(self)
        }
    }
}

impl FromStr for PackageUrl<'static> {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Parse all components into strings (since we don't know infer from `s` lifetime)
        let (s, _) = parser::parse_scheme(s)?;
        let (s, subpath) = parser::parse_subpath(s)?;
        let (s, ql) = parser::parse_qualifiers(s)?;
        let (s, version) = parser::parse_version(s)?;
        let (s, ty) = parser::parse_type(s)?;
        let (s, mut name) = parser::parse_name(s)?;
        let (_, mut namespace) = parser::parse_namespace(s)?;

        // Special rules for some types
        match ty.as_ref() {
            "bitbucket" | "github" | "composer" => {
                name = name.to_lowercase();
                namespace = namespace.map(|ns| ns.to_lowercase());
            }
            "pypi" => {
                name = name.replace('_', "-").to_lowercase();
            }
            "cpan" => {
                if namespace.is_none() {
                    return Err(Error::MissingNamespace);
                }
            }
            "cran" => {
                if version.is_none() {
                    return Err(Error::MissingVersion);
                }
            }
            _ => {}
        };

        let mut purl = Self::new(ty, name)?;
        if let Some(ns) = namespace {
            purl.with_namespace(ns);
        }
        if let Some(v) = version {
            purl.with_version(v);
        }
        if let Some(sp) = subpath {
            purl.with_subpath(sp)?;
        }
        for (k, v) in ql.into_iter() {
            purl.add_qualifier(k, v)?;
        }

        // The obtained package url
        Ok(purl)
    }
}

impl Display for PackageUrl<'_> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        // Scheme: constant
        f.write_str("pkg:")?;

        // Type: no encoding needed
        self.ty.fmt(f).and(f.write_str("/"))?;

        // Namespace: percent-encode each component
        if let Some(ref ns) = self.namespace {
            for component in ns.split('/').map(|s| s.encode(ENCODE_SET)) {
                component.fmt(f).and(f.write_str("/"))?;
            }
        }

        // Name: percent-encode the name
        self.name.encode(ENCODE_SET).fmt(f)?;

        // Version: percent-encode the version
        if let Some(ref v) = self.version {
            f.write_str("@").and(v.encode(ENCODE_SET).fmt(f))?;
        }

        // Qualifiers: percent-encode the values
        if !self.qualifiers.is_empty() {
            f.write_str("?")?;

            let mut items = self.qualifiers.iter().collect::<Vec<_>>();
            items.sort();

            let mut iter = items.into_iter();
            if let Some((k, v)) = iter.next() {
                k.fmt(f)
                    .and(f.write_str("="))
                    .and(v.encode(ENCODE_SET).fmt(f))?;
            }
            for (k, v) in iter {
                f.write_str("&")
                    .and(k.fmt(f))
                    .and(f.write_str("="))
                    .and(v.encode(ENCODE_SET).fmt(f))?;
            }
        }

        // Subpath: percent-encode the components
        if let Some(ref sp) = self.subpath {
            f.write_str("#")?;
            let mut components = sp
                .split('/')
                .filter(|&s| !(s.is_empty() || s == "." || s == ".."));
            if let Some(component) = components.next() {
                component.encode(ENCODE_SET).fmt(f)?;
            }
            for component in components {
                f.write_str("/")?;
                component.encode(ENCODE_SET).fmt(f)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from_str() {
        let raw_purl = "pkg:type/name/space/name@version?k1=v1&k2=v2#sub/path";
        let purl = PackageUrl::from_str(raw_purl).unwrap();
        assert_eq!(purl.ty(), "type");
        assert_eq!(purl.namespace(), Some("name/space"));
        assert_eq!(purl.name(), "name");
        assert_eq!(purl.version(), Some("version"));
        assert_eq!(purl.qualifiers().get("k1"), Some(&Cow::Borrowed("v1")));
        assert_eq!(purl.qualifiers().get("k2"), Some(&Cow::Borrowed("v2")));
        assert_eq!(purl.subpath(), Some("sub/path"));
    }

    #[test]
    fn test_to_str() {
        let canonical = "pkg:type/name/space/name@version?k1=v1&k2=v2#sub/path";
        let purl_string = PackageUrl::new("type", "name")
            .unwrap()
            .with_namespace("name/space")
            .with_version("version")
            .with_subpath("sub/path")
            .unwrap()
            .add_qualifier("k1", "v1")
            .unwrap()
            .add_qualifier("k2", "v2")
            .unwrap()
            .to_string();
        assert_eq!(&purl_string, canonical);
    }

    #[test]
    fn test_percent_encoding_idempotent() {
        let orig = "pkg:brew/openssl%25401.1@1.1.1w";
        let round_trip = orig.parse::<PackageUrl>().unwrap().to_string();
        assert_eq!(orig, round_trip);
    }

    #[test]
    fn test_percent_encoding_qualifier() {
        let mut purl = "pkg:deb/ubuntu/gnome-calculator@1:41.1-2ubuntu2"
            .parse::<PackageUrl>()
            .unwrap();
        purl.add_qualifier(
            "vcs_url",
            "git+https://salsa.debian.org/gnome-team/gnome-calculator.git@debian/1%41.1-2",
        )
        .unwrap();
        let encoded = purl.to_string();
        assert_eq!(encoded, "pkg:deb/ubuntu/gnome-calculator@1:41.1-2ubuntu2?vcs_url=git%2Bhttps:%2F%2Fsalsa.debian.org%2Fgnome-team%2Fgnome-calculator.git%40debian%2F1%2541.1-2");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        let mut purl = PackageUrl::new("type", "name").unwrap();
        purl.with_namespace("name/space")
            .with_version("version")
            .with_subpath("sub/path")
            .unwrap()
            .add_qualifier("k1", "v1")
            .unwrap()
            .add_qualifier("k2", "v2")
            .unwrap();

        let j = serde_json::to_string(&purl).unwrap();
        let purl2: PackageUrl = serde_json::from_str(&j).unwrap();

        assert_eq!(purl, purl2);
    }

    #[test]
    fn test_plus_sign_in_version() {
        let expected = "pkg:type/name@1%2Bx";
        for purl in [
            "pkg:type/name@1+x",
            "pkg:type/name@1%2bx",
            "pkg:type/name@1%2Bx",
        ] {
            let actual = PackageUrl::from_str(purl).unwrap().to_string();
            assert_eq!(actual, expected);
        }
    }
}
