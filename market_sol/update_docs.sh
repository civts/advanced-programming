#! /usr/bin/env bash

if [ $# -ne 1 ]; then
  echo "Missing kellnr token. Please specify it as the first (and only) argument to this script"
  exit 1
fi

CARGO_REGISTRIES_KELLNR_TOKEN=${1}

echo "This script generates an updated version of the crate documentation and pushes it to kellnr"

# Remove old documentation
rm -rf ./target/doc.zip ./target/doc || true

# Generate new documentation
cargo doc

PACKAGE_NAME=$(cat Cargo.toml | grep "^name = " | head -1 |  awk  '{print $3}' | sed 's/"//g')
VERSION_NUMBER=$(cat Cargo.toml | grep "^version = " | head -1 |  awk  '{print $3}' | sed 's/"//g')

cd ./target

# Zip the docs to prepare them for uploading
zip -r doc.zip ./doc

# Update the docs on kellnr
curl -H "Authorization: $CARGO_REGISTRIES_KELLNR_TOKEN" \
  "http://advancedprogramming.disi.unitn.it:8000/api/v1/docs/${PACKAGE_NAME}/${VERSION_NUMBER}" \
  --upload-file doc.zip

rm doc.zip
