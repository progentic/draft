set shell := ["bash", "-cu"]

default: verify

bootstrap:
    bash scripts/bootstrap.sh

format:
    bash scripts/format.sh

verify:
    bash scripts/verify.sh

check-invariants:
    bash scripts/check-invariants.sh

docs-check:
    bash scripts/check-docs.sh

build:
    bash scripts/build.sh
