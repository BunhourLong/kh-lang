#!/usr/bin/env bash
# Fetch SIL khmerlbdict word-frequency sources (MIT) used by build-dict.mjs.
# Usage: bash scripts/fetch-dict.sh   (writes scripts/data-source/)
set -euo pipefail
cd "$(dirname "$0")"
commit="54abc6b87f4ffa6a0c95c09836c7c86648002273"
base="https://raw.githubusercontent.com/silnrsi/khmerlbdict/${commit}/src"
mkdir -p data-source
for f in seafreq KHOV KHSV DFD HC TD names places villages; do
  curl -fsSL "$base/$f.txt" -o "data-source/$f.txt"
  echo "fetched $f.txt"
done
curl -fsSL "https://raw.githubusercontent.com/silnrsi/khmerlbdict/${commit}/LICENSE" -o ../licenses/khmerlbdict-LICENSE
node build-dict.mjs
