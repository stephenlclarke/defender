.PHONY: fmt test clippy release-gate ci ci-smoke ci-doctor coverage-doctor smoke-doctor coverage sq sq-ci sonar run run-wgpu live live-wgpu smoke-wgpu actor-smoke actor-attract-smoke actor-post-game-smoke actor-wgpu-smoke live-smoke readme-gameplay-image readme-attract-sequence readme-media docs-lint diff-check clean

SONAR_SCANNER ?= sonar-scanner
SONAR_ARGS ?= -Dsonar.qualitygate.wait=true
XVFB_RUN ?= xvfb-run
VULKANINFO ?= vulkaninfo
DOCS_MARKDOWN := README.md
README_GAMEPLAY_IMAGE ?= docs/defender.png
README_ATTRACT_SEQUENCE ?= docs/start-sequence.gif

fmt:
	cargo fmt --check

test:
	cargo test --all-targets

clippy:
	cargo clippy --all-targets -- -D warnings

release-gate:
	$(MAKE) fmt
	$(MAKE) test
	$(MAKE) clippy
	$(MAKE) actor-smoke
	$(MAKE) actor-attract-smoke
	$(MAKE) actor-post-game-smoke
	$(MAKE) actor-wgpu-smoke
	$(MAKE) live-smoke
	$(MAKE) readme-media
	$(MAKE) docs-lint
	$(MAKE) diff-check

ci: fmt test clippy coverage docs-lint diff-check

ci-smoke: smoke-wgpu actor-wgpu-smoke

ci-doctor: coverage-doctor smoke-doctor

coverage-doctor:
	@cargo llvm-cov --version >/dev/null 2>&1 || { \
		echo "cargo-llvm-cov is required. Install with: cargo install cargo-llvm-cov"; \
		exit 1; \
	}
	@rustup component list --installed | grep -q '^llvm-tools-' || { \
		echo "llvm-tools-preview is required. Install with: rustup component add llvm-tools-preview"; \
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

coverage: coverage-doctor
	mkdir -p target/coverage
	rustup run stable cargo llvm-cov --all-targets --workspace --fail-under-lines 80 --lcov --output-path target/coverage/lcov.info
	rustup run stable cargo llvm-cov report --cobertura --output-path target/coverage/coverage.xml

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

run-wgpu: run

live: run

live-wgpu: run-wgpu

smoke-wgpu:
	cargo run -- --live-smoke

actor-smoke:
	cargo run -- --actor-smoke

actor-attract-smoke:
	cargo run -- --actor-attract-smoke

actor-post-game-smoke:
	cargo run -- --actor-post-game-smoke

actor-wgpu-smoke:
	cargo run -- --actor-wgpu-smoke

live-smoke: smoke-wgpu

readme-gameplay-image:
	cargo run --quiet --example generate_readme_media -- --gameplay "$(README_GAMEPLAY_IMAGE)"

readme-attract-sequence:
	cargo run --quiet --example generate_readme_media -- --attract "$(README_ATTRACT_SEQUENCE)"

readme-media:
	cargo run --quiet --example generate_readme_media -- --gameplay "$(README_GAMEPLAY_IMAGE)" --attract "$(README_ATTRACT_SEQUENCE)"

docs-lint:
	markdownlint $(DOCS_MARKDOWN)

diff-check:
	git diff --check

clean:
	cargo clean
	rm -rf .scannerwork
