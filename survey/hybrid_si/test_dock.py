"""Tests of the docking harness's patcher and parsers; nothing here compiles or runs C++."""
import pathlib, sys, unittest

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


if __name__ == "__main__":
    unittest.main()
