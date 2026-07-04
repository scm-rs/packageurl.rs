#!/bin/sh
# Vendors the purl-spec conformance suite into tests/purl-spec/.
#
# Source: https://github.com/package-url/purl-spec at the commit pinned below (MIT,
# the upstream LICENSE is vendored alongside the files). The suite is fetched at
# exactly that commit and every file is verified against scripts/purl-spec-tests.sha256,
# so a re-run either reproduces the vendored tree bit for bit or fails.
#
# Bumping the suite: edit PIN, run with --refresh to rewrite the manifest from the
# fresh checkout, review the diff, then re-baseline the known gaps:
# PURL_CONFORMANCE_DUMP=1 cargo test --test purl_conformance -- --nocapture
set -eu

PIN=995f3878f5bb6979bc6560cd747e422e8262f18b
REPO=https://github.com/package-url/purl-spec.git

ROOT=$(git -C "$(dirname "$0")" rev-parse --show-toplevel)
DEST="$ROOT/tests/purl-spec"
MANIFEST="$ROOT/scripts/purl-spec-tests.sha256"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

git init -q "$tmp/purl-spec"
git -C "$tmp/purl-spec" fetch -q --depth 1 "$REPO" "$PIN"
git -C "$tmp/purl-spec" checkout -q FETCH_HEAD

rm -rf "$DEST"
mkdir -p "$DEST/spec" "$DEST/types"
cp "$tmp/purl-spec/LICENSE" "$DEST/LICENSE"
cp "$tmp/purl-spec/tests/spec/"*.json "$DEST/spec/"
cp "$tmp/purl-spec/tests/types/"*.json "$DEST/types/"

cd "$ROOT"
if [ "${1:-}" = "--refresh" ]; then
    find tests/purl-spec -type f | LC_ALL=C sort | xargs sha256sum > "$MANIFEST"
    echo "wrote $(wc -l < "$MANIFEST" | tr -d ' ') digests to scripts/purl-spec-tests.sha256"
fi
sha256sum --check --quiet "$MANIFEST"
echo "vendored the purl-spec test suite at $PIN"
