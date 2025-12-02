#![cfg_attr(rustfmt, rustfmt_skip)]

#[macro_use]
mod macros;
mod testcase;


generate_json_tests! {
    alpm_test          => "tests/spec/purl-spec/tests/types/alpm-test.json",
    apk_test           => "tests/spec/purl-spec/tests/types/apk-test.json",
    bintray_test       => "tests/spec/purl-spec/tests/types/bintray-test.json",
    bitbucket_test     => "tests/spec/purl-spec/tests/types/bitbucket-test.json",
    bitnami_test       => "tests/spec/purl-spec/tests/types/bitnami-test.json",
    cargo_test         => "tests/spec/purl-spec/tests/types/cargo-test.json",
    cocoapods_test     => "tests/spec/purl-spec/tests/types/cocoapods-test.json",
    composer_test      => "tests/spec/purl-spec/tests/types/composer-test.json",
    conan_test         => "tests/spec/purl-spec/tests/types/conan-test.json",
    conda_test         => "tests/spec/purl-spec/tests/types/conda-test.json",
    cpan_test          => "tests/spec/purl-spec/tests/types/cpan-test.json",
    cran_test          => "tests/spec/purl-spec/tests/types/cran-test.json",
    deb_test           => "tests/spec/purl-spec/tests/types/deb-test.json",
    docker_test        => "tests/spec/purl-spec/tests/types/docker-test.json",
    gem_test           => "tests/spec/purl-spec/tests/types/gem-test.json",
    generic_test       => "tests/spec/purl-spec/tests/types/generic-test.json",
    github_test        => "tests/spec/purl-spec/tests/types/github-test.json",
    golang_test        => "tests/spec/purl-spec/tests/types/golang-test.json",
    hackage_test       => "tests/spec/purl-spec/tests/types/hackage-test.json",
    hex_test           => "tests/spec/purl-spec/tests/types/hex-test.json",
    huggingface_test   => "tests/spec/purl-spec/tests/types/huggingface-test.json",
    luarocks_test      => "tests/spec/purl-spec/tests/types/luarocks-test.json",
    maven_test         => "tests/spec/purl-spec/tests/types/maven-test.json",
    mlflow_test        => "tests/spec/purl-spec/tests/types/mlflow-test.json",
    npm_test           => "tests/spec/purl-spec/tests/types/npm-test.json",
    nuget_test         => "tests/spec/purl-spec/tests/types/nuget-test.json",
    oci_test           => "tests/spec/purl-spec/tests/types/oci-test.json",
    pub_test           => "tests/spec/purl-spec/tests/types/pub-test.json",
    pypi_test          => "tests/spec/purl-spec/tests/types/pypi-test.json",
    qpkg_test          => "tests/spec/purl-spec/tests/types/qpkg-test.json",
    rpm_test           => "tests/spec/purl-spec/tests/types/rpm-test.json",
    swid_test          => "tests/spec/purl-spec/tests/types/swid-test.json",
    // swift_test         => "tests/spec/purl-spec/tests/types/swift-test.json",
}

