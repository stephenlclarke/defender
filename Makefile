.PHONY: fmt test legacy-tools-test clippy legacy-tools-clippy fidelity release-gate clean-fidelity ci ci-doctor trace-doctor coverage-doctor smoke-doctor reference-media-doctor reference-mame-doctor trace-script-test media-script-test trace-fixtures reference-inputs reference-traces reference-fixtures-check reference-media-fetch reference-mame-capture reference-mame-smoke reference-clean-capture reference-window-scan reference-window-scan-organic reference-report-gate reference-signoff-summary reference-evidence-package owner-review-package reference-media-check coverage coverage-new-code coverage-new-code-baseline sq sq-ci sonar run run-wgpu live live-wgpu smoke-wgpu game-smoke live-smoke docs-lint diff-check readme-media

SONAR_SCANNER ?= sonar-scanner
SONAR_ARGS ?= -Dsonar.qualitygate.wait=true
FIDELITY_TRACE_FIXTURES ?= docs/fidelity/fixtures/local/rust-current
PHASE_ONE_SCENARIOS := \
	attract_boot \
	start_game \
	first_300_frames \
	firing \
	thrust_reverse \
	smart_bomb \
	hyperspace \
	abduction \
	death
SCENARIOS ?= $(PHASE_ONE_SCENARIOS)
DEFENDER_MAME ?= mame
DEFENDER_ROM_DIR ?= assets/roms
DEFENDER_REFERENCE_TRACE_DIR ?= docs/fidelity/fixtures/local/reference
NEW_CODE_COVERAGE_BASE ?=
NEW_CODE_COVERAGE_EFFECTIVE_BASE = $(or $(strip $(NEW_CODE_COVERAGE_BASE)),HEAD)
NEW_CODE_COVERAGE_BASELINE ?= tools/new_rust_coverage_baseline.txt
LUA ?= lua
PYTHON ?= python3
README_START_SEQUENCE_GIF ?= target/readme-media/start-sequence-candidate.gif
README_START_SEQUENCE_WAV ?= target/readme-media/start-sequence-candidate.wav
REFERENCE_MEDIA_URL ?= https://youtu.be/gss3lxeqCok
REFERENCE_MEDIA_LOCAL ?= target/reference-media/sources/defender-red-label-mame-gss3lxeqCok.mp4
REFERENCE_MEDIA ?= $(REFERENCE_MEDIA_LOCAL)
REFERENCE_AUDIO ?=
REFERENCE_MEDIA_REPORT_DIR ?= target/reference-media
REFERENCE_MEDIA_VISUAL_FPS ?= 6
REFERENCE_MEDIA_REFERENCE_START_MS ?=
REFERENCE_MEDIA_CANDIDATE_START_MS ?=
REFERENCE_MEDIA_DURATION_MS ?=
REFERENCE_MEDIA_REPORT_ONLY ?=
REFERENCE_MEDIA_ACCEPTANCE_MODE ?= all
CANDIDATE_MEDIA ?= $(README_START_SEQUENCE_GIF)
CANDIDATE_AUDIO ?= $(README_START_SEQUENCE_WAV)
MAME_REFERENCE_DIR ?= target/reference-media/mame
MAME_REFERENCE_SECONDS ?= 60
MAME_REFERENCE_BASENAME ?= defender-red-label-golden-$(MAME_REFERENCE_SECONDS)s
MAME_REFERENCE_SMOKE_SECONDS ?= 2
MAME_REFERENCE_SMOKE_BASENAME ?= defender-red-label-smoke-script
MAME_REFERENCE_TRACE_ONLY ?=
MAME_REFERENCE_STATE_STEER ?=
MAME_REFERENCE_STATE_STEER_FRAME ?= 1400
REFERENCE_WINDOW_SCAN_ROOT ?= $(MAME_REFERENCE_DIR)
REFERENCE_WINDOW_SCAN_REPORT ?= target/reference-media/reference-window-scan.json
REFERENCE_WINDOW_SCAN_ORGANIC_REPORT ?= target/reference-media/reference-window-scan-organic.json
REFERENCE_WINDOW_SCAN_PROXIMITY ?= 24
REFERENCE_WINDOW_SCAN_EXCLUDES ?=
REFERENCE_WINDOW_SCAN_EXCLUDE_ARGS = $(foreach fragment,$(REFERENCE_WINDOW_SCAN_EXCLUDES),--exclude-path-fragment "$(fragment)")
REFERENCE_WINDOW_SCAN_ORGANIC_EXCLUDES ?= nonlander-sound-command enemy-explosion-matrix enemy-materialize-matrix state-steered
REFERENCE_WINDOW_SCAN_ORGANIC_EXCLUDE_ARGS = $(foreach fragment,$(REFERENCE_WINDOW_SCAN_ORGANIC_EXCLUDES),--exclude-path-fragment "$(fragment)")
REFERENCE_REPORT_MANIFEST ?= docs/fidelity/reference-report-gate.json
REFERENCE_SIGNOFF_SUMMARY ?= target/reference-media/reference-signoff-summary.md
CLEAN_REFERENCE_DIR ?= target/reference-media/clean
REFERENCE_SCENARIO ?=
REFERENCE_INPUT_PROGRAM ?=
CLEAN_REFERENCE_BASENAME ?= defender-clean-candidate
CLEAN_REFERENCE_SAMPLE_STEP ?= 6
CLEAN_REFERENCE_STATE_STEER ?=
CLEAN_REFERENCE_STATE_STEER_FRAME ?= 1400
CLEAN_REFERENCE_CAPTURE_START_FRAME ?=
CLEAN_REFERENCE_CAPTURE_END_FRAME ?=
FFMPEG ?= ffmpeg
YTDLP ?= yt-dlp
YOUTUBEDR ?= youtubedr
XVFB_RUN ?= xvfb-run
VULKANINFO ?= vulkaninfo
DOCS_MARKDOWN := README.md SPEC.md PLAN.md docs/fidelity/mame-golden-clips.md docs/fidelity/release-closure-audit.md assets/sounds/README.md

fmt:
	cargo fmt --check

test:
	cargo test --all-targets

clippy:
	cargo clippy --all-targets -- -D warnings

legacy-tools-test:
	cargo test --all-targets --features legacy-tools

legacy-tools-clippy:
	cargo clippy --all-targets --features legacy-tools -- -D warnings

fidelity: fmt test clippy legacy-tools-test legacy-tools-clippy trace-script-test media-script-test trace-fixtures coverage

release-gate:
	$(MAKE) fmt
	$(MAKE) test
	$(MAKE) legacy-tools-test
	$(MAKE) clippy
	$(MAKE) legacy-tools-clippy
	$(MAKE) clean-fidelity
	$(MAKE) media-script-test
	$(MAKE) owner-review-package
	$(MAKE) reference-mame-doctor
	$(MAKE) reference-mame-smoke
	$(MAKE) readme-media
	$(MAKE) game-smoke
	$(MAKE) live-smoke
	$(MAKE) docs-lint
	$(MAKE) diff-check

clean-fidelity:
	CLEAN_FIDELITY_SCENARIOS="$(SCENARIOS)" cargo test --lib --features legacy-tools clean_fidelity_reports_selected_scenarios -- --ignored --nocapture

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

reference-media-doctor:
	@command -v $(PYTHON) >/dev/null 2>&1 || { \
		echo "Python interpreter '$(PYTHON)' is required for reference media verification."; \
		exit 1; \
	}
	@command -v $(FFMPEG) >/dev/null 2>&1 || { \
		echo "$(FFMPEG) is required for reference media verification. Install ffmpeg."; \
		exit 1; \
	}

reference-mame-doctor: reference-media-doctor
	@command -v $(DEFENDER_MAME) >/dev/null 2>&1 || { \
		echo "$(DEFENDER_MAME) is required for MAME reference capture. Install MAME or set DEFENDER_MAME."; \
		exit 1; \
	}
	@"$(DEFENDER_MAME)" -rompath "$(DEFENDER_ROM_DIR)" -verifyroms defender >/dev/null

trace-script-test: trace-doctor
	$(LUA) -v >/dev/null
	DEFENDER_TRACE_SELF_TEST=1 $(LUA) tools/mame_defender_trace.lua
	$(PYTHON) -m unittest tools/generate_reference_traces_test.py
	$(PYTHON) -m unittest tools/check_new_rust_coverage_test.py

media-script-test:
	$(PYTHON) -m unittest tools/verify_reference_media_test.py
	$(PYTHON) -m unittest tools/capture_mame_reference_test.py
	$(PYTHON) -m unittest tools/scan_reference_windows_test.py
	$(PYTHON) -m unittest tools/check_reference_reports_test.py

trace-fixtures:
	cargo run --quiet --features legacy-tools -- --fidelity-check-trace-dir "$(FIDELITY_TRACE_FIXTURES)"

reference-inputs:
	cargo run --quiet --features legacy-tools -- --fidelity-write-scenario-inputs "$(DEFENDER_REFERENCE_TRACE_DIR)"

reference-traces:
	$(PYTHON) tools/generate_reference_traces.py --mame "$(DEFENDER_MAME)" --rom-dir "$(DEFENDER_ROM_DIR)" --out-dir "$(DEFENDER_REFERENCE_TRACE_DIR)"

reference-fixtures-check:
	cargo run --quiet --features legacy-tools -- --fidelity-check-reference-trace-dir "$(DEFENDER_REFERENCE_TRACE_DIR)"

reference-media-fetch:
	@mkdir -p "$$(dirname "$(REFERENCE_MEDIA_LOCAL)")"
	@if command -v "$(YTDLP)" >/dev/null 2>&1; then \
		"$(YTDLP)" --no-playlist --format 'bv*[height<=720]+ba/b[height<=720]/best[height<=720]/best' --merge-output-format mp4 --paths "$$(dirname "$(REFERENCE_MEDIA_LOCAL)")" --output "$$(basename "$(REFERENCE_MEDIA_LOCAL)" .mp4).%(ext)s" "$(REFERENCE_MEDIA_URL)"; \
	elif command -v "$(YOUTUBEDR)" >/dev/null 2>&1; then \
		"$(YOUTUBEDR)" download --quality medium --mimetype mp4 --directory "$$(dirname "$(REFERENCE_MEDIA_LOCAL)")" --filename "$$(basename "$(REFERENCE_MEDIA_LOCAL)")" "$(REFERENCE_MEDIA_URL)"; \
	else \
		echo "$(YTDLP) or $(YOUTUBEDR) is required to fetch reference media."; \
		exit 1; \
	fi

reference-mame-capture: reference-mame-doctor
	$(PYTHON) tools/capture_mame_reference.py \
		--mame "$(DEFENDER_MAME)" \
		--ffmpeg "$(FFMPEG)" \
		--rom-dir "$(DEFENDER_ROM_DIR)" \
		--out-dir "$(MAME_REFERENCE_DIR)" \
		--seconds "$(MAME_REFERENCE_SECONDS)" \
		--basename "$(MAME_REFERENCE_BASENAME)" \
		$(if $(strip $(MAME_REFERENCE_TRACE_ONLY)),--trace-only,) \
		$(if $(strip $(MAME_REFERENCE_STATE_STEER)),--state-steer "$(MAME_REFERENCE_STATE_STEER)",) \
		$(if $(strip $(MAME_REFERENCE_STATE_STEER)),--state-steer-frame "$(MAME_REFERENCE_STATE_STEER_FRAME)",) \
		$(if $(strip $(REFERENCE_SCENARIO)),--scenario "$(REFERENCE_SCENARIO)",) \
		$(if $(strip $(REFERENCE_INPUT_PROGRAM)),--input-program "$(REFERENCE_INPUT_PROGRAM)",)

reference-mame-smoke:
	$(MAKE) reference-mame-capture \
		MAME_REFERENCE_SECONDS="$(MAME_REFERENCE_SMOKE_SECONDS)" \
		MAME_REFERENCE_BASENAME="$(MAME_REFERENCE_SMOKE_BASENAME)"

reference-clean-capture:
	cargo run --quiet --features legacy-tools --example generate_reference_candidate_media -- \
		--out-dir "$(CLEAN_REFERENCE_DIR)" \
		--basename "$(CLEAN_REFERENCE_BASENAME)" \
		--sample-step "$(CLEAN_REFERENCE_SAMPLE_STEP)" \
		$(if $(strip $(CLEAN_REFERENCE_STATE_STEER)),--state-steer "$(CLEAN_REFERENCE_STATE_STEER)",) \
		$(if $(strip $(CLEAN_REFERENCE_STATE_STEER)),--state-steer-frame "$(CLEAN_REFERENCE_STATE_STEER_FRAME)",) \
		$(if $(strip $(CLEAN_REFERENCE_CAPTURE_START_FRAME)),--capture-start-frame "$(CLEAN_REFERENCE_CAPTURE_START_FRAME)",) \
		$(if $(strip $(CLEAN_REFERENCE_CAPTURE_END_FRAME)),--capture-end-frame "$(CLEAN_REFERENCE_CAPTURE_END_FRAME)",) \
		$(if $(strip $(REFERENCE_SCENARIO)),--scenario "$(REFERENCE_SCENARIO)",) \
		$(if $(strip $(REFERENCE_INPUT_PROGRAM)),--input-program "$(REFERENCE_INPUT_PROGRAM)",)

reference-window-scan:
	$(PYTHON) tools/scan_reference_windows.py \
		--root "$(REFERENCE_WINDOW_SCAN_ROOT)" \
		--object-proximity-frames "$(REFERENCE_WINDOW_SCAN_PROXIMITY)" \
		--out-json "$(REFERENCE_WINDOW_SCAN_REPORT)" \
		$(REFERENCE_WINDOW_SCAN_EXCLUDE_ARGS)

reference-window-scan-organic:
	$(PYTHON) tools/scan_reference_windows.py \
		--root "$(REFERENCE_WINDOW_SCAN_ROOT)" \
		--object-proximity-frames "$(REFERENCE_WINDOW_SCAN_PROXIMITY)" \
		--out-json "$(REFERENCE_WINDOW_SCAN_ORGANIC_REPORT)" \
		$(REFERENCE_WINDOW_SCAN_ORGANIC_EXCLUDE_ARGS)

reference-report-gate:
	$(PYTHON) tools/check_reference_reports.py \
		--manifest "$(REFERENCE_REPORT_MANIFEST)"

reference-signoff-summary:
	$(PYTHON) tools/check_reference_reports.py \
		--manifest "$(REFERENCE_REPORT_MANIFEST)" \
		--summary-out "$(REFERENCE_SIGNOFF_SUMMARY)"

reference-evidence-package: reference-window-scan reference-window-scan-organic reference-signoff-summary

owner-review-package: reference-evidence-package reference-report-gate
	@printf '\nOwner review package ready:\n'
	@printf '  %s\n' "$(REFERENCE_SIGNOFF_SUMMARY)"
	@printf '  %s\n' "docs/fidelity/release-closure-audit.md"
	@printf '\nOwner review checklist:\n'
	@awk '/^## Owner Review Checklist/ { active = 1; next } /^## Current Conclusion/ { active = 0 } active { print }' docs/fidelity/release-closure-audit.md

reference-media-check: reference-media-doctor
	@test -n "$(strip $(REFERENCE_MEDIA))" || { \
		echo "REFERENCE_MEDIA must point to a local captured reference video."; \
		exit 1; \
	}
	$(PYTHON) tools/verify_reference_media.py \
		--ffmpeg "$(FFMPEG)" \
		--reference-video "$(REFERENCE_MEDIA)" \
		$(if $(strip $(REFERENCE_AUDIO)),--reference-audio "$(REFERENCE_AUDIO)",) \
		--candidate-media "$(CANDIDATE_MEDIA)" \
		$(if $(strip $(CANDIDATE_AUDIO)),--candidate-audio "$(CANDIDATE_AUDIO)",) \
		--out-dir "$(REFERENCE_MEDIA_REPORT_DIR)" \
		--visual-fps "$(REFERENCE_MEDIA_VISUAL_FPS)" \
		$(if $(strip $(REFERENCE_MEDIA_REFERENCE_START_MS)),--reference-start-ms "$(REFERENCE_MEDIA_REFERENCE_START_MS)",) \
		$(if $(strip $(REFERENCE_MEDIA_CANDIDATE_START_MS)),--candidate-start-ms "$(REFERENCE_MEDIA_CANDIDATE_START_MS)",) \
		$(if $(strip $(REFERENCE_MEDIA_DURATION_MS)),--duration-ms "$(REFERENCE_MEDIA_DURATION_MS)",) \
		--acceptance-mode "$(REFERENCE_MEDIA_ACCEPTANCE_MODE)" \
		$(if $(strip $(REFERENCE_MEDIA_REPORT_ONLY)),--report-only,)

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

run-wgpu: run

live: run

live-wgpu: run-wgpu

smoke-wgpu:
	cargo run -- --live-smoke

game-smoke:
	cargo run -- --game-smoke

live-smoke: smoke-wgpu

docs-lint:
	markdownlint $(DOCS_MARKDOWN)

diff-check:
	git diff --check

readme-media:
	cargo run --quiet --features legacy-tools --example generate_readme_media -- "$(README_START_SEQUENCE_GIF)" --audio "$(README_START_SEQUENCE_WAV)"
