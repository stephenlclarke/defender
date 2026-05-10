import unittest
from pathlib import Path
import sys


sys.path.insert(0, str(Path(__file__).resolve().parent))

from check_new_rust_coverage import (
    executable_added_lines,
    parse_added_rust_lines,
    parse_lcov_line_counts,
    production_added_lines,
    uncovered_added_lines,
)


class CheckNewRustCoverageTest(unittest.TestCase):
    def test_parse_added_rust_lines_tracks_new_side_numbers(self):
        repo_root = Path("/repo")
        diff = """diff --git a/src/app.rs b/src/app.rs
--- a/src/app.rs
+++ b/src/app.rs
@@ -10,0 +11,2 @@
+let covered = true;
+let uncovered = false;
@@ -20 +23,0 @@
-let removed = true;
"""

        added = parse_added_rust_lines(diff, repo_root)

        self.assertEqual(added, {Path("/repo/src/app.rs"): {11, 12}})

    def test_parse_added_rust_lines_can_ignore_moved_lines(self):
        repo_root = Path("/repo")
        diff = """diff --git a/src/machine.rs b/src/machine.rs
--- a/src/machine.rs
+++ b/src/machine.rs
@@ -10,2 +10,0 @@
-    let moved = true;
-    let duplicated = 1;
@@ -30,0 +29,3 @@
+    let moved = true;
+    let duplicated = 1;
+    let new_line = true;
"""

        added = parse_added_rust_lines(diff, repo_root, ignore_moved=True)

        self.assertEqual(added, {Path("/repo/src/machine.rs"): {31}})

    def test_parse_added_rust_lines_treats_visibility_only_moves_as_moved(self):
        repo_root = Path("/repo")
        diff = """diff --git a/src/machine.rs b/src/machine.rs
--- a/src/machine.rs
+++ b/src/machine.rs
@@ -10 +10,0 @@
-fn helper() -> bool {
diff --git a/src/machine_child.rs b/src/machine_child.rs
new file mode 100644
--- /dev/null
+++ b/src/machine_child.rs
@@ -0,0 +1,2 @@
+pub(super) fn helper() -> bool {
+    true
"""

        added = parse_added_rust_lines(diff, repo_root, ignore_moved=True)

        self.assertEqual(added, {Path("/repo/src/machine_child.rs"): {2}})

    def test_parse_lcov_line_counts_normalizes_relative_sources(self):
        repo_root = Path("/repo")
        lcov = """TN:
SF:src/app.rs
DA:11,1
DA:12,0
end_of_record
"""

        coverage = parse_lcov_line_counts(lcov, repo_root)

        self.assertEqual(coverage, {Path("/repo/src/app.rs"): {11: 1, 12: 0}})

    def test_uncovered_added_lines_ignores_non_executable_added_lines(self):
        path = Path("/repo/src/app.rs")
        added = {path: {11, 12, 13}}
        coverage = {path: {11: 1, 12: 0}}

        instrumented, uncovered = uncovered_added_lines(added, coverage)

        self.assertEqual(instrumented, 2)
        self.assertEqual(uncovered, [(path, 12)])

    def test_production_added_lines_ignores_cfg_test_module_tail(self):
        repo_root = Path(self.create_temp_dir())
        source = repo_root / "src" / "app.rs"
        source.parent.mkdir()
        source.write_text("fn production() {}\n#[cfg(test)]\nmod tests {}\n", encoding="utf-8")

        self.assertEqual(production_added_lines({source: {1, 3}}), {source: {1}})

    def test_production_added_lines_keeps_production_after_test_module(self):
        repo_root = Path(self.create_temp_dir())
        source = repo_root / "src" / "app.rs"
        source.parent.mkdir()
        source.write_text(
            "fn before() {}\n"
            "#[cfg(test)]\n"
            "mod tests {\n"
            "    #[test]\n"
            "    fn it_works() {\n"
            "        assert_eq!(\"{\", \"{\");\n"
            "    }\n"
            "}\n"
            "fn after() {}\n",
            encoding="utf-8",
        )

        self.assertEqual(
            production_added_lines({source: {1, 4, 6, 9}}),
            {source: {1, 9}},
        )

    def test_production_added_lines_ignores_multiline_cfg_test_item(self):
        repo_root = Path(self.create_temp_dir())
        source = repo_root / "src" / "live.rs"
        source.parent.mkdir()
        source.write_text(
            "fn production() {}\n"
            "#[cfg(test)]\n"
            "pub fn run_live(\n"
            "    _play_audio: bool,\n"
            "    _cmos_path: Option<&Path>,\n"
            ") -> Result<()> {\n"
            "    Ok(())\n"
            "}\n"
            "fn after() {}\n",
            encoding="utf-8",
        )

        self.assertEqual(
            production_added_lines({source: {1, 3, 4, 5, 6, 7, 9}}),
            {source: {1, 9}},
        )

    def test_production_added_lines_ignores_cfg_test_or_coverage_stub_only(self):
        repo_root = Path(self.create_temp_dir())
        source = repo_root / "src" / "live.rs"
        source.parent.mkdir()
        source.write_text(
            "#[cfg(all(not(test), not(coverage)))]\n"
            "pub fn production_live() {\n"
            "    terminal_loop();\n"
            "}\n"
            "#[cfg(any(test, coverage))]\n"
            "pub fn production_live() {\n"
            "    Ok(())\n"
            "}\n",
            encoding="utf-8",
        )

        self.assertEqual(
            production_added_lines({source: {1, 2, 3, 5, 6, 7}}),
            {source: {1, 2, 3}},
        )

    def test_executable_added_lines_ignores_structural_rust_lines(self):
        repo_root = Path(self.create_temp_dir())
        source = repo_root / "src" / "machine.rs"
        source.parent.mkdir()
        source.write_text(
            "fn production() -> Result<(), String> {\n"
            "    work(\n"
            "        value,\n"
            "    )?;\n"
            "    Ok(())\n"
            "}\n",
            encoding="utf-8",
        )

        self.assertEqual(
            executable_added_lines({source: {2, 3, 4, 5, 6}}),
            {source: {2, 3, 5}},
        )

    def create_temp_dir(self):
        import shutil
        import tempfile

        path = tempfile.mkdtemp()
        self.addCleanup(shutil.rmtree, path)
        return path


if __name__ == "__main__":
    unittest.main()
