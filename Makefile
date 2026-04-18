.PHONY: fmt test clippy ci coverage

fmt:
	cargo fmt --check

test:
	cargo test --all-targets

clippy:
	cargo clippy --all-targets -- -D warnings

ci: fmt test clippy

coverage:
	mkdir -p target/coverage
	cargo llvm-cov --all-targets --workspace --cobertura --output-path target/coverage/coverage.xml
