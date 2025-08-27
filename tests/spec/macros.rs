use crate::spec::testcase::SpecTestCase;
use crate::spec::testcase::TestSuite;
use crate::spec::testcase::PurlOrString;
use std::path::Path;
use std::fs;
use std::borrow::Cow;
use packageurl::PackageUrl;
use std::str::FromStr;


pub fn run_parse_test(case: &SpecTestCase) {
    if let PurlOrString::String(input) = &case.input {
        if let Ok(purl) = PackageUrl::from_str(input) {
            assert!(!case.expected_failure, "Expected failure: but parsing succeeded for PURL: {}", input);

            if let Some(PurlOrString::PurlComponent(expected)) = &case.expected_output {
                assert_eq!(Some(purl.ty()), expected.ty.as_ref().map(Cow::as_ref));
                assert_eq!(Some(purl.name()), expected.name.as_ref().map(Cow::as_ref));
                assert_eq!(purl.namespace(), expected.namespace.as_ref().map(Cow::as_ref));
                assert_eq!(purl.version(), expected.version.as_ref().map(Cow::as_ref));
                assert_eq!(purl.subpath(), expected.subpath.as_ref().map(Cow::as_ref));

                if let Some(ref expected_quals) = expected.qualifiers {
                    assert_eq!(purl.qualifiers(), expected_quals);
                } else {
                    assert!(purl.qualifiers().is_empty());
                }
            } else {
                panic!("Expected PurlComponent as expected_output for: {}", case.description);
            }
        } else {
            assert!(case.expected_failure, "Unexpected parse failure: {}", case.description);
        }
    }
}


pub fn run_build_test(case: &SpecTestCase) {
    let PurlOrString::PurlComponent(ref input) = case.input else {
        panic!("Expected PurlComponent as input for build test: {}", case.description);
    };

    if input.ty.is_none() || input.name.is_none() {
        assert!(case.expected_failure, "Missing type or name, but test not marked as failure: {}", case.description);
        return;
    }

    let ty = input.ty.as_ref().unwrap().as_ref();
    let name = input.name.as_ref().unwrap().as_ref();

    let purl_result = PackageUrl::new(ty, name);

    if purl_result.is_err() {
        assert!(case.expected_failure, "Purl build failed: {}", case.description);
        return;
    }

    let mut purl = purl_result.unwrap();
    if let Some(ref ns) = input.namespace {
        purl.with_namespace(ns.as_ref());
    }

    if let Some(ref v) = input.version {
        purl.with_version(v.as_ref());
    }

    if let Some(ref sp) = input.subpath {
        purl.with_subpath(sp.as_ref()).unwrap();
    }

    if let Some(ref quals) = input.qualifiers {
        for (k, v) in quals.iter() {
            if purl.add_qualifier(k.as_ref(), v.as_ref()).is_err() {
                assert!(case.expected_failure, "add_qualifier failed unexpectedly");
                return;
            }
        }
    }

    assert!(!case.expected_failure, "Test was expected to fail but succeeded: {}", case.description);
    if let Some(PurlOrString::String(expected)) = &case.expected_output {
        assert_eq!(&purl.to_string(), expected);
    } else {
        panic!("Expected String as expected_output for build test: {}", case.description);
    }
}

pub fn run_roundtrip_test(case: &SpecTestCase) {
    let input = match &case.input {
        PurlOrString::String(s) => s,
        _ => panic!("Input must be a string: {}", case.description),
    };

    if let Ok(purl) = PackageUrl::from_str(input) {
        assert!(!case.expected_failure, "Test was expected to fail but succeeded: {}", case.description);
        if let Some(PurlOrString::String(expected)) = &case.expected_output {
            assert_eq!(&purl.to_string(), expected);
        }
    } else {
        assert!(case.expected_failure, "Failed to create PURL for: {}", input);
    }
}


pub fn run_tests_from_spec(path: &Path) {
    let data = fs::read(path).expect("Failed to read test file");
    let suite: TestSuite = serde_json::from_slice(&data).expect("Invalid test file");

    for case in suite.tests {

        match case.test_type.as_ref() {
            "parse" => {
               run_parse_test(&case);
            }
            "build" => {
                run_build_test(&case);
            }
            "roundtrip" => {
                run_roundtrip_test(&case);
            }
            other => {
                println!("Unknown test type '{}', skipping: {}", other, case.description);
            }
        }
    }
}

#[macro_export]
macro_rules! generate_json_tests {
    ($($test_name:ident => $file_path:expr),* $(,)?) => {
        $(
            #[test]
            fn $test_name() {
                crate::spec::macros::run_tests_from_spec(std::path::Path::new($file_path));
            }
        )*
    };
}
