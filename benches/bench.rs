use criterion::{Criterion, criterion_group, criterion_main};
use packageurl::PackageUrl;
use std::str::FromStr;

pub fn bench_from_str(c: &mut Criterion) {
    let raw_purl = "pkg:type/name/space/name@version?k1=v1&k2=v2#sub/path";
    c.bench_function("from_str", |c| {
        c.iter(|| {
            let _ = PackageUrl::from_str(raw_purl).unwrap();
        })
    });
}

pub fn bench_from_url(c: &mut Criterion) {
    let raw_purl = "pkg:type/name/space/name@version?k1=v1&k2=v2#sub/path";
    c.bench_function("parse", |c| {
        c.iter(|| {
            let _ = url::Url::parse(raw_purl).unwrap();
        })
    });
}

criterion_group! {
  benches, bench_from_str, bench_from_url
}
criterion_main!(benches);
