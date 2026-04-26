.PHONY: fmt test clippy fidelity ci trace-fixtures reference-inputs reference-traces reference-fixtures-check coverage sq sq-ci sonar run run-muted live live-muted readme-media

SONAR_SCANNER ?= sonar-scanner
SONAR_ARGS ?= -Dsonar.qualitygate.wait=true
FIDELITY_TRACE_FIXTURES ?= docs/fidelity/fixtures/local
DEFENDER_MAME ?= mame
DEFENDER_ROM_DIR ?= assets/roms
DEFENDER_REFERENCE_TRACE_DIR ?= docs/fidelity/fixtures/local

fmt:
	cargo fmt --check

test:
	cargo test --all-targets

clippy:
	cargo clippy --all-targets -- -D warnings

fidelity: fmt test clippy trace-fixtures coverage

ci: fidelity

trace-fixtures:
	cargo run --quiet -- --fidelity-check-trace-dir "$(FIDELITY_TRACE_FIXTURES)"

reference-inputs:
	cargo run --quiet -- --fidelity-write-scenario-inputs "$(DEFENDER_REFERENCE_TRACE_DIR)"

reference-traces:
	python3 tools/generate_reference_traces.py --mame "$(DEFENDER_MAME)" --rom-dir "$(DEFENDER_ROM_DIR)" --out-dir "$(DEFENDER_REFERENCE_TRACE_DIR)"

reference-fixtures-check:
	cargo run --quiet -- --fidelity-check-reference-trace-dir "$(DEFENDER_REFERENCE_TRACE_DIR)"

coverage:
	@cargo llvm-cov --version >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@rustup component list --installed | grep -q '^llvm-tools-' || { \
		echo "llvm-tools-preview is required. Install with: rustup component add llvm-tools-preview"; \
		exit 1; \
	}
	mkdir -p target/coverage
	rustup run stable cargo llvm-cov --all-targets --workspace --fail-under-lines 80 --cobertura --output-path target/coverage/coverage.xml

sq-ci: coverage

sq:
	@test -n "$$SONAR_TOKEN" || { \
		echo "SONAR_TOKEN must be set in the environment."; \
		exit 1; \
	}
	@command -v $(SONAR_SCANNER) >/dev/null 2>&1 || { \
		echo "$(SONAR_SCANNER) is required for local SonarQube scans."; \
		exit 1; \
	}
	@$(MAKE) coverage
	$(SONAR_SCANNER) $(SONAR_ARGS)

sonar: sq

run:
	cargo run

run-muted:
	cargo run -- --mute

live: run

live-muted: run-muted

readme-media:
	@echo "README media generation is archived under oldsrc/examples during the clean-slate rewrite."
