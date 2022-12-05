ARCH := x86_64-apple-darwin

build:
	cargo build --release --target=${ARCH}
	rm -rf target/workflow/${ARCH}
	mkdir -p target/workflow/${ARCH}
	cp target/${ARCH}/release/alfred-chrome config/icon.png README.md target/workflow/${ARCH}
	cp target/workflow/${ARCH}/icon.png target/workflow/${ARCH}/211C5A45-1E27-402E-AF94-671ECF92C23D.png
	cp target/workflow/${ARCH}/icon.png target/workflow/${ARCH}/73168DC1-FA1E-4B36-93F1-2F1039E75721.png
	yq '.readme = load_str("config/README.md")' config/workflow.json -P -o json | plutil -convert xml1 -o target/workflow/${ARCH}/info.plist -
	rm -rf dist/${ARCH}/Alfred\ Chrome.alfredworkflow
	mkdir -p dist/${ARCH}/
	zip -jr dist/${ARCH}/Alfred\ Chrome.alfredworkflow target/workflow/${ARCH}

lint:
	cargo +nightly fmt --check

check:
	cargo clippy -- -D warnings

default: build

.PHONY: build lint check