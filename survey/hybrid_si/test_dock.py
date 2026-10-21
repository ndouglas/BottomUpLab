"""Tests of the docking harness's patcher, parsers and runner; nothing here compiles or runs C++
(the runner's tests replace subprocess.run)."""
import pathlib, subprocess, sys, tempfile, unittest
from unittest import mock

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import dock  # noqa: E402

WELL_MIXED = """c#include <iostream>
int loop = 100;  // Number of loops
int main(void)
{
    sgenrand(time(0));  // seed
    for(r = 0.00; r <= 0.21 ; r += 0.01){
        for(rho = 0.0; rho <= 0.5; rho += 0.01) {
            double record_fc[loop]={0};
        }
    }
}
"""
LATTICE = """#include <iostream>
#define loop 100 // 100 loops
int main(void) {
    sgenrand(time(0));
    for (r = 0; r < 0.21; r += 0.01) {
        for (rho = 0; rho < 0.51; rho += 0.01) {
        }
    }
}
"""


class Patches(unittest.TestCase):
    def test_well_mixed_source_is_pinned_to_one_point(self):
        out, applied = dock.patch(WELL_MIXED)
        self.assertTrue(out.startswith("#include <iostream>"))
        self.assertIn("const int loop = SI_LOOP;", out)
        self.assertIn("sgenrand(SI_SEED);", out)
        self.assertIn("for (r = SI_R; r <= SI_R; r += 1)", out)
        self.assertIn("for (rho = SI_RHO; rho <= SI_RHO; rho += 1)", out)
        self.assertEqual([a["name"] for a in applied], ["stray characters before #include", "constant run count", "seed", "r loop", "rho loop"])

    def test_lattice_source_keeps_its_define(self):
        out, applied = dock.patch(LATTICE)
        self.assertIn("#define loop SI_LOOP // 100 loops", out)
        self.assertNotIn("stray characters before #include", [a["name"] for a in applied])

    def test_a_missing_pattern_is_an_error(self):
        with self.assertRaises(ValueError):
            dock.patch(LATTICE.replace("sgenrand(time(0));", ""))


class Parsers(unittest.TestCase):
    def test_well_mixed_lines(self):
        out = "r,rho,Kappa,theta,loop,c_fraction\n0.050000,0.300000,10.000000,0.900000,0,0.123400\n0.050000,0.300000,10.000000,0.900000,1,0.234500\n"
        self.assertEqual(dock.parse("well_mixed", "discrete", out), [(0.05, 0.3, 0.1234), (0.05, 0.3, 0.2345)])

    def test_lattice_lines_by_framework(self):
        out = "*****start*****\n0.100000\t0.500000\t0.612300\t0.387700\n"
        self.assertEqual(dock.parse("lattice", "mixed", out), [(0.1, 0.5, 0.6123)])
        cont = "0.100000\t0.500000\t0.900000\t0.555000\t0.445000\n"
        self.assertEqual(dock.parse("lattice", "continuous", cont), [(0.1, 0.5, 0.555)])

    def test_points_and_seeds(self):
        points = dock.points()
        self.assertEqual(len(points), 9 + 27)
        self.assertEqual(points[0], ("fig1a", 0.05, 0.3))
        self.assertEqual(dock.seeds(0), [1, 2, 3, 4])
        self.assertEqual(dock.seeds(35), [141, 142, 143, 144])

    def test_csv_has_one_row_per_run(self):
        results = [("fig3a", 0.1, 0.5, 1, 6, [0.25], 1.0), ("fig1a", 0.05, 0.3, 0, 1, [0.5, 0.125], 2.0)]
        self.assertEqual(dock.csv("fig1a", results), "r,rho,chunk,seed,run,cooperation\n0.05,0.3,0,1,0,0.500000\n0.05,0.3,0,1,1,0.125000\n")
        self.assertIsNone(dock.csv("fig1b", results))

    def test_smoke_points_are_each_files_first_point(self):
        self.assertEqual(dock.smoke_indices(), [0, 3, 6, 9, 18, 27])
        self.assertEqual([dock.points()[p][0] for p in dock.smoke_indices()], list(dock.FILES))


class Runner(unittest.TestCase):
    LINE = "r,rho,Kappa,theta,loop,c_fraction\n0.050000,0.300000,10.000000,0.900000,0,0.500000\n"

    def fake_run(self, argv, **kw):
        self.calls.append((argv, kw))
        return subprocess.CompletedProcess(argv, 0, stdout=self.LINE if argv[0] != "c++" else "", stderr="")

    def run_once(self, scratch):
        job = ("fig1a", 0.05, 0.3, 1, 1, "int main(){}", scratch, 0)
        with mock.patch.object(dock.subprocess, "run", side_effect=self.fake_run):
            return dock.run_chunk(job)

    def test_the_binary_runs_only_under_sandbox_exec(self):
        self.calls = []
        with tempfile.TemporaryDirectory() as t:
            out = self.run_once(pathlib.Path(t))
            argv, kw = self.calls[1]
            box = kw["cwd"]
            self.assertEqual(argv, ["/usr/bin/sandbox-exec", "-D", f"BOX={box}", "-p", dock.SANDBOX, "./run"])
            self.assertEqual(box, box.resolve())
            self.assertEqual(self.calls[0][1]["cwd"], box)
            self.assertEqual(out[5], [0.5])

    def test_a_finished_chunk_is_resumed_not_rerun(self):
        self.calls = []
        with tempfile.TemporaryDirectory() as t:
            first = self.run_once(pathlib.Path(t))
            self.calls = []
            second = self.run_once(pathlib.Path(t))
            self.assertEqual(self.calls, [])
            self.assertEqual(first[:6], second[:6])

    def test_links_left_in_an_old_box_are_not_followed(self):
        # A previous binary may write anything inside its box, including links to files outside it.
        # The fake compiler writes box/run the way the linker would, following any link there.
        def compile_like_a_linker(argv, **kw):
            self.calls.append((argv, kw))
            if argv[0] == "c++":
                (kw["cwd"] / "run").write_text("binary", encoding="utf-8")
            return subprocess.CompletedProcess(argv, 0, stdout=self.LINE if argv[0] != "c++" else "", stderr="")

        for planted in ("x.cpp", "run"):
            with self.subTest(planted=planted), tempfile.TemporaryDirectory() as t:
                self.calls = []
                scratch = pathlib.Path(t).resolve()
                sentinel = scratch / "sentinel.txt"
                sentinel.write_text("untouched", encoding="utf-8")
                box = scratch / "fig1a-r0.05-rho0.3-seed1" / "box"
                box.mkdir(parents=True)
                (box / planted).symlink_to(sentinel)
                job = ("fig1a", 0.05, 0.3, 1, 1, "int main(){}", scratch, 0)
                with mock.patch.object(dock.subprocess, "run", side_effect=compile_like_a_linker):
                    dock.run_chunk(job)
                self.assertEqual(sentinel.read_text(encoding="utf-8"), "untouched")
                self.assertFalse((box / planted).is_symlink())

        # The box itself replaced by a link to a directory outside it (BOX's own path is writable).
        with tempfile.TemporaryDirectory() as t:
            self.calls = []
            scratch = pathlib.Path(t).resolve()
            elsewhere = scratch / "elsewhere"
            elsewhere.mkdir()
            (elsewhere / "x.cpp").write_text("untouched", encoding="utf-8")
            home = scratch / "fig1a-r0.05-rho0.3-seed1"
            home.mkdir()
            (home / "box").symlink_to(elsewhere, target_is_directory=True)
            with mock.patch.object(dock.subprocess, "run", side_effect=compile_like_a_linker):
                dock.run_chunk(("fig1a", 0.05, 0.3, 1, 1, "int main(){}", scratch, 0))
            self.assertEqual(sorted(p.name for p in elsewhere.iterdir()), ["x.cpp"])
            self.assertEqual((elsewhere / "x.cpp").read_text(encoding="utf-8"), "untouched")
            self.assertFalse((home / "box").is_symlink())

    def test_a_failure_reports_its_stderr(self):
        def failing(argv, **kw):
            raise subprocess.CalledProcessError(1, argv, output="", stderr="x.cpp:3: error: boom")
        with tempfile.TemporaryDirectory() as t, mock.patch.object(dock.subprocess, "run", side_effect=failing):
            with self.assertRaisesRegex(RuntimeError, "boom"):
                dock.run_chunk(("fig1a", 0.05, 0.3, 1, 1, "int main(){}", pathlib.Path(t), 0))


if __name__ == "__main__":
    unittest.main()
