.DEFAULT_GOAL := help

.PHONY: help fmt test clippy release-gate ci ci-smoke ci-doctor coverage-doctor smoke-doctor coverage sq sq-ci sonar run run-wgpu live live-wgpu smoke smoke-wgpu actor-smoke actor-attract-smoke actor-post-game-smoke actor-wgpu-smoke live-smoke readme-gameplay-image readme-attract-sequence readme-media readme-media-check docs-lint diff-check clean

CARGO ?= cargo
RUSTUP ?= rustup
COVERAGE_TOOLCHAIN ?= stable
SONAR_SCANNER ?= sonar-scanner
SONAR_ARGS ?= -Dsonar.qualitygate.wait=true
XVFB_RUN ?= xvfb-run
VULKANINFO ?= vulkaninfo
UNAME_S ?= $(shell uname -s)
DOCS_MARKDOWN := README.md
README_GAMEPLAY_IMAGE ?= docs/defender.png
README_ATTRACT_SEQUENCE ?= docs/start-sequence.gif

help:
	@printf '%s\n' \
		"Targets:" \
		"  make run                         Launch the playable game" \
		"  make fmt                         Check Rust formatting" \
		"  make test                        Run all Rust tests" \
		"  make clippy                      Run clippy with warnings denied" \
		"  make smoke                       Run actor and WGPU smoke checks" \
		"  make release-gate                Run local release checks without rewriting README media" \
		"  make ci                          Run non-graphical CI checks and coverage" \
		"  make ci-smoke                    Run CI smoke checks; wrap with xvfb-run on Linux CI" \
		"  make ci-doctor                   Check CI coverage and smoke prerequisites" \
		"  make coverage                    Generate lcov and Cobertura coverage reports" \
		"  make sq-ci                       Generate SonarCloud coverage artifacts" \
		"  make sq                          Run local Sonar scan after coverage" \
		"  make readme-media                Regenerate committed README media" \
		"  make readme-media-check          Verify committed README media is current" \
		"  make docs-lint                   Lint Markdown docs" \
		"  make diff-check                  Check whitespace in the working diff" \
		"  make clean                       Remove generated build/scanner artifacts"

fmt:
	$(CARGO) fmt --check

test:
	$(CARGO) test --all-targets

clippy:
	$(CARGO) clippy --all-targets -- -D warnings

release-gate: fmt test clippy smoke readme-media-check docs-lint diff-check

ci: fmt test clippy coverage docs-lint diff-check

ci-smoke: smoke-doctor smoke-wgpu actor-wgpu-smoke

ci-doctor: coverage-doctor smoke-doctor

coverage-doctor:
	@$(RUSTUP) run $(COVERAGE_TOOLCHAIN) rustc --version >/dev/null 2>&1 || { \
		echo "Rust toolchain '$(COVERAGE_TOOLCHAIN)' is required. Install with: rustup toolchain install $(COVERAGE_TOOLCHAIN)"; \
		exit 1; \
	}
	@$(RUSTUP) run $(COVERAGE_TOOLCHAIN) $(CARGO) llvm-cov --version >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required for toolchain '$(COVERAGE_TOOLCHAIN)'. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@$(RUSTUP) component list --installed --toolchain $(COVERAGE_TOOLCHAIN) | grep -q '^llvm-tools' || { \
		echo "llvm-tools-preview is required for toolchain '$(COVERAGE_TOOLCHAIN)'. Install with: rustup component add llvm-tools-preview --toolchain $(COVERAGE_TOOLCHAIN)"; \
		exit 1; \
	}

smoke-doctor:
	@if [ "$(UNAME_S)" = "Linux" ]; then \
		command -v $(XVFB_RUN) >/dev/null 2>&1 || { \
			echo "$(XVFB_RUN) is required for headless wgpu smoke in Linux CI. Install xvfb."; \
			exit 1; \
		}; \
		command -v $(VULKANINFO) >/dev/null 2>&1 || { \
			echo "$(VULKANINFO) is required to diagnose Mesa/Vulkan availability. Install vulkan-tools."; \
			exit 1; \
		}; \
		LIBGL_ALWAYS_SOFTWARE=1 $(VULKANINFO) --summary >/dev/null || { \
			echo "Mesa/Vulkan software renderer check failed; verify mesa-vulkan-drivers and libasound2-dev."; \
			exit 1; \
		}; \
	else \
		echo "smoke-doctor: skipping Linux display/Vulkan checks on $(UNAME_S)."; \
	fi

coverage: coverage-doctor
	mkdir -p target/coverage
	$(RUSTUP) run $(COVERAGE_TOOLCHAIN) $(CARGO) llvm-cov --all-targets --workspace --fail-under-lines 80 --lcov --output-path target/coverage/lcov.info
	$(RUSTUP) run $(COVERAGE_TOOLCHAIN) $(CARGO) llvm-cov report --cobertura --output-path target/coverage/coverage.xml

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
	$(CARGO) run

run-wgpu: run

live: run

live-wgpu: run-wgpu

smoke: actor-smoke actor-attract-smoke actor-post-game-smoke actor-wgpu-smoke live-smoke

smoke-wgpu:
	$(CARGO) run -- --live-smoke

actor-smoke:
	$(CARGO) run -- --actor-smoke

actor-attract-smoke:
	$(CARGO) run -- --actor-attract-smoke

actor-post-game-smoke:
	$(CARGO) run -- --actor-post-game-smoke

actor-wgpu-smoke:
	$(CARGO) run -- --actor-wgpu-smoke

live-smoke: smoke-wgpu

readme-gameplay-image:
	$(CARGO) run --quiet --example generate_readme_media -- --gameplay "$(README_GAMEPLAY_IMAGE)"

readme-attract-sequence:
	$(CARGO) run --quiet --example generate_readme_media -- --attract "$(README_ATTRACT_SEQUENCE)"

readme-media:
	$(CARGO) run --quiet --example generate_readme_media -- --gameplay "$(README_GAMEPLAY_IMAGE)" --attract "$(README_ATTRACT_SEQUENCE)"

readme-media-check:
	@tmp_dir=$$(mktemp -d); \
	trap 'rm -rf "$$tmp_dir"' EXIT; \
	$(CARGO) run --quiet --example generate_readme_media -- --gameplay "$$tmp_dir/defender.png" --attract "$$tmp_dir/start-sequence.gif"; \
	cmp -s "$(README_GAMEPLAY_IMAGE)" "$$tmp_dir/defender.png" || { \
		echo "$(README_GAMEPLAY_IMAGE) is stale. Run: make readme-media"; \
		exit 1; \
	}; \
	cmp -s "$(README_ATTRACT_SEQUENCE)" "$$tmp_dir/start-sequence.gif" || { \
		echo "$(README_ATTRACT_SEQUENCE) is stale. Run: make readme-media"; \
		exit 1; \
	}

docs-lint:
	markdownlint $(DOCS_MARKDOWN)

diff-check:
	git diff --check

clean:
	cargo clean
	rm -rf .scannerwork
