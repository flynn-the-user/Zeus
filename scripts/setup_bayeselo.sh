#!/usr/bin/env bash
# Fetches and builds the bayeselo CLI (Rémi Coulom's GPL-licensed rating
# tool) into tools/bayeselo/bayeselo. See Reference/testing/BayesElo.md.
# Not vendored into the repo (tools/ is gitignored) - this script is the
# reproducible install step.
set -euo pipefail
cd "$(dirname "$0")/.."

mkdir -p tools
if [ ! -d tools/BayesianElo ]; then
    git clone --depth 1 https://github.com/ddugovic/BayesianElo.git tools/BayesianElo
fi

cd tools/BayesianElo/src
# The 2010-era source predates C++11's stricter rules on string-literal/
# macro concatenation; -std=gnu++98 builds the unmodified source as-is
# instead of patching it.
g++ -o bayeselo -O3 -std=gnu++98 bayeselo.cpp

mkdir -p ../../bayeselo
cp bayeselo ../../bayeselo/bayeselo
echo "Built tools/bayeselo/bayeselo"
