#!/bin/bash

CARGO=cargo

set -ex

build() {
    "$CARGO" build --target "$TARGET" --release

    # Output most recent build error
    set +x
    stderr="$(find "target/$TARGET/release" -name stderr -print0 |
        xargs -0 ls -t | head -n1)"
    if [ -s "$stderr" ]; then
        echo "===== $stderr ====="
        cat "$stderr"
        echo "====="
    fi
    set -x
}

make_tarball() {
    local name="${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}"
    local temp_dir="$(mktemp -d)"
    local stage_dir="$temp_dir/$name"
    local out_dir="$(pwd)/deploy"
    mkdir -p "$out_dir"

    cp "target/$TARGET/release/frece" "$stage_dir/"
    cp LICENSE "$stage_dir/"
    cp README.md "$stage_dir/"

    (cd "$temp_dir" && tar czf "$out_dir/$name.tar.gz" "$name")
    rm -rf "$temp_dir"
}

main() {
    build
    make_tarball
}

main
