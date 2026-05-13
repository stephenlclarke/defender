.PHONY: fmt test clippy fidelity ci ci-doctor trace-doctor coverage-doctor smoke-doctor trace-script-test trace-fixtures reference-inputs reference-traces reference-fixtures-check coverage coverage-new-code coverage-new-code-baseline sq sq-ci sonar run run-wgpu run-kitty live live-wgpu live-kitty smoke-wgpu readme-media

SONAR_SCANNER ?= sonar-scanner
SONAR_ARGS ?= -Dsonar.qualitygate.wait=true
FIDELITY_TRACE_FIXTURES ?= docs/fidelity/fixtures/local/rust-current
DEFENDER_MAME ?= mame
DEFENDER_ROM_DIR ?= assets/roms
DEFENDER_REFERENCE_TRACE_DIR ?= docs/fidelity/fixtures/local/reference
NEW_CODE_COVERAGE_BASE ?=
NEW_CODE_COVERAGE_EFFECTIVE_BASE = $(or $(strip $(NEW_CODE_COVERAGE_BASE)),HEAD)
NEW_CODE_COVERAGE_BASELINE ?= tools/new_rust_coverage_baseline.txt
LUA ?= lua
PYTHON ?= python3
README_START_SEQUENCE_GIF ?= docs/start-sequence.gif
XVFB_RUN ?= xvfb-run
VULKANINFO ?= vulkaninfo

fmt:
	cargo fmt --check

test:
	cargo test --all-targets

clippy:
	cargo clippy --all-targets -- -D warnings

fidelity: fmt test clippy trace-script-test trace-fixtures coverage

ci: fidelity smoke-wgpu

ci-doctor: trace-doctor coverage-doctor smoke-doctor

trace-doctor:
	@command -v $(LUA) >/dev/null 2>&1 || { \
		echo "Lua interpreter '$(LUA)' is required. On Ubuntu CI install lua5.4 and run with LUA=lua5.4."; \
		exit 1; \
	}
	@$(LUA) -v >/dev/null
	@command -v $(PYTHON) >/dev/null 2>&1 || { \
		echo "Python interpreter '$(PYTHON)' is required for trace and coverage helper tests."; \
		exit 1; \
	}

coverage-doctor:
	@cargo llvm-cov --version >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@rustup component list --installed | grep -q '^llvm-tools-' || { \
		echo "llvm-tools-preview is required. Install with: rustup component add llvm-tools-preview"; \
		exit 1; \
	}
	@test -z "$(strip $(NEW_CODE_COVERAGE_BASELINE))" || test -f "$(NEW_CODE_COVERAGE_BASELINE)" || { \
		echo "NEW_CODE_COVERAGE_BASELINE file is missing: $(NEW_CODE_COVERAGE_BASELINE)"; \
		exit 1; \
	}

smoke-doctor:
	@command -v $(XVFB_RUN) >/dev/null 2>&1 || { \
		echo "$(XVFB_RUN) is required for headless wgpu smoke in Linux CI. Install xvfb."; \
		exit 1; \
	}
	@command -v $(VULKANINFO) >/dev/null 2>&1 || { \
		echo "$(VULKANINFO) is required to diagnose Mesa/Vulkan availability. Install vulkan-tools."; \
		exit 1; \
	}
	@LIBGL_ALWAYS_SOFTWARE=1 $(VULKANINFO) --summary >/dev/null || { \
		echo "Mesa/Vulkan software renderer check failed; verify mesa-vulkan-drivers and libasound2-dev."; \
		exit 1; \
	}

trace-script-test: trace-doctor
	$(LUA) -v >/dev/null
	DEFENDER_TRACE_SELF_TEST=1 $(LUA) tools/mame_defender_trace.lua
	$(PYTHON) -m unittest tools/generate_reference_traces_test.py
	$(PYTHON) -m unittest tools/check_new_rust_coverage_test.py

trace-fixtures:
	cargo run --quiet -- --fidelity-check-trace-dir "$(FIDELITY_TRACE_FIXTURES)"

reference-inputs:
	cargo run --quiet -- --fidelity-write-scenario-inputs "$(DEFENDER_REFERENCE_TRACE_DIR)"

reference-traces:
	$(PYTHON) tools/generate_reference_traces.py --mame "$(DEFENDER_MAME)" --rom-dir "$(DEFENDER_ROM_DIR)" --out-dir "$(DEFENDER_REFERENCE_TRACE_DIR)"

reference-fixtures-check:
	cargo run --quiet -- --fidelity-check-reference-trace-dir "$(DEFENDER_REFERENCE_TRACE_DIR)"

coverage: coverage-doctor
	mkdir -p target/coverage
	rustup run stable cargo llvm-cov --all-targets --workspace --fail-under-lines 80 --lcov --output-path target/coverage/lcov.info
	$(PYTHON) tools/check_new_rust_coverage.py --lcov target/coverage/lcov.info --base "$(NEW_CODE_COVERAGE_EFFECTIVE_BASE)" $(if $(strip $(NEW_CODE_COVERAGE_BASELINE)),--uncovered-baseline "$(NEW_CODE_COVERAGE_BASELINE)",)
	rustup run stable cargo llvm-cov report --cobertura --output-path target/coverage/coverage.xml

coverage-new-code:
	@test -n "$(NEW_CODE_COVERAGE_BASE)" || { \
		echo "NEW_CODE_COVERAGE_BASE must be set, for example: make coverage-new-code NEW_CODE_COVERAGE_BASE=origin/main"; \
		exit 1; \
	}
	@test -f target/coverage/lcov.info || { \
		echo "target/coverage/lcov.info is missing; run make coverage first"; \
		exit 1; \
	}
	$(PYTHON) tools/check_new_rust_coverage.py --lcov target/coverage/lcov.info --base "$(NEW_CODE_COVERAGE_BASE)" $(if $(strip $(NEW_CODE_COVERAGE_BASELINE)),--uncovered-baseline "$(NEW_CODE_COVERAGE_BASELINE)",)

coverage-new-code-baseline:
	@test -n "$(NEW_CODE_COVERAGE_BASE)" || { \
		echo "NEW_CODE_COVERAGE_BASE must be set, for example: make coverage-new-code-baseline NEW_CODE_COVERAGE_BASE=origin/main"; \
		exit 1; \
	}
	@test -n "$(NEW_CODE_COVERAGE_BASELINE)" || { \
		echo "NEW_CODE_COVERAGE_BASELINE must be set."; \
		exit 1; \
	}
	@test -f target/coverage/lcov.info || { \
		echo "target/coverage/lcov.info is missing; run make coverage first"; \
		exit 1; \
	}
	$(PYTHON) tools/check_new_rust_coverage.py --lcov target/coverage/lcov.info --base "$(NEW_CODE_COVERAGE_BASE)" --write-uncovered-baseline "$(NEW_CODE_COVERAGE_BASELINE)"

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

run-wgpu:
	cargo run -- --renderer wgpu

run-kitty:
	cargo run -- --renderer kitty

live: run

live-wgpu: run-wgpu

live-kitty: run-kitty

smoke-wgpu:
	cargo run -- --live-smoke

readme-media:
	cargo run --quiet --example generate_readme_media -- "$(README_START_SEQUENCE_GIF)"
