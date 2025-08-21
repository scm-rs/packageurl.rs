macro_rules! spec_tests {
    ($name:ident, $desc:expr) => {
        mod $name {

            use super::testcase::SpecTestCase;
            use packageurl::PackageUrl;
            use std::borrow::Cow;
            use std::str::FromStr;
            use std::sync::LazyLock;

            static TEST_CASE: LazyLock<SpecTestCase<'static>> =
                LazyLock::new(|| SpecTestCase::new($desc));

            #[test]
            fn purl_to_components() {
                if let Ok(purl) = PackageUrl::from_str(&TEST_CASE.purl) {
                    assert!(!TEST_CASE.is_invalid);
                    assert_eq!(TEST_CASE.ty.as_ref().unwrap().as_ref(), purl.ty());
                    assert_eq!(TEST_CASE.name.as_ref().unwrap().as_ref(), purl.name());
                    assert_eq!(
                        TEST_CASE.namespace.as_ref().map(Cow::as_ref),
                        purl.namespace()
                    );
                    assert_eq!(TEST_CASE.version.as_ref().map(Cow::as_ref), purl.version());
                    assert_eq!(TEST_CASE.subpath.as_ref().map(Cow::as_ref), purl.subpath());
                    if let Some(ref quals) = TEST_CASE.qualifiers {
                        assert_eq!(quals, purl.qualifiers());
                    } else {
                        assert!(purl.qualifiers().is_empty());
                    }
                } else {
                    assert!(TEST_CASE.is_invalid);
                }
            }

            #[test]
            fn components_to_canonical() {
                if TEST_CASE.is_invalid {
                    return;
                }

                let mut purl = PackageUrl::new(
                    TEST_CASE.ty.as_ref().unwrap().clone(),
                    TEST_CASE.name.as_ref().unwrap().clone(),
                )
                .unwrap();

                if let Some(ref ns) = TEST_CASE.namespace {
                    purl.with_namespace(ns.as_ref()).unwrap();
                }

                if let Some(ref v) = TEST_CASE.version {
                    purl.with_version(v.as_ref()).unwrap();
                }

                if let Some(ref sp) = TEST_CASE.subpath {
                    purl.with_subpath(sp.as_ref()).unwrap();
                }

                if let Some(ref quals) = TEST_CASE.qualifiers {
                    for (k, v) in quals.iter() {
                        purl.add_qualifier(k.as_ref(), v.as_ref()).unwrap();
                    }
                }

                assert_eq!(
                    TEST_CASE.canonical_purl.as_ref().unwrap(),
                    &purl.to_string()
                );
            }

            #[test]
            fn canonical_to_canonical() {
                if TEST_CASE.is_invalid {
                    return;
                }

                let purl =
                    PackageUrl::from_str(&TEST_CASE.canonical_purl.as_ref().unwrap()).unwrap();
                assert_eq!(
                    TEST_CASE.canonical_purl.as_ref().unwrap(),
                    &purl.to_string()
                );
            }

            #[test]
            fn purl_to_canonical() {
                if TEST_CASE.is_invalid {
                    return;
                }
                let purl = PackageUrl::from_str(&TEST_CASE.purl).unwrap();
                assert_eq!(
                    TEST_CASE.canonical_purl.as_ref().unwrap(),
                    &purl.to_string()
                )
            }
        }
    };
}
