"""Closed source authentication for lean delivery; no engine or entropy."""
from pathlib import Path
import tempfile
import unittest
from survey.war1_study import registration

ROOT = Path(__file__).resolve().parents[2]
CORE = 'crates/sugarscape-core/'

class LeanIdentityTests(unittest.TestCase):
    def fixture(self, root):
        def put(path, text):
            file = root / path
            file.parent.mkdir(parents=True, exist_ok=True)
            file.write_text(text)
        put(registration.LITERAL_INVENTORY, 'data/anasazi/Map.txt\n')
        put('data/anasazi/Map.txt', 'map')
        put(CORE + 'src/identity_bytes.rs', (ROOT / (CORE + 'src/identity_bytes.rs')).read_text())
        put(CORE + 'src/lib.rs', 'include_str!("../../../data/anasazi/Map.txt");\nidentity_bytes!("payload.rs");\n')
        put(CORE + 'src/payload.rs', '// payload\n')
        return put

    def test_actual_current_literal_inventory_remains_complete(self):
        self.assertEqual(len(registration.audit_literal_inputs(ROOT)), 253)

    def test_identity_macro_preserves_native_input_and_rejects_forged_expansion(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            self.fixture(root)
            self.assertEqual(registration.audit_literal_inputs(root), {'data/anasazi/Map.txt'})
            path = root / (CORE + 'src/lib.rs')
            good = path.read_text()
            for call in ['identity_bytes!(env!("INPUT"))', 'identity_bytes!("/tmp/forged")',
                         'identity_bytes!("../../../../escape")', 'identity_bytes!("missing.rs")',
                         'identity_bytes/**/!("payload.rs")', 'identity_bytes!(concat!("payload", env!("INPUT")))']:
                with self.subTest(call=call):
                    path.write_text(good.replace('identity_bytes!("payload.rs")', call))
                    with self.assertRaises((ValueError, OSError)):
                        registration.audit_literal_inputs(root)
            path.write_text(good)
            macro = root / (CORE + 'src/identity_bytes.rs')
            macro.write_text(macro.read_text().replace('b"".as_slice()', 'b"forged".as_slice()'))
            with self.assertRaises(ValueError):
                registration.audit_literal_inputs(root)

    def test_only_bound_generated_outputs_are_accepted_and_host_is_required(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            put = self.fixture(root)
            output = 'include!(concat!(env!("OUT_DIR"), "/compact_raw_receipts.rs"))'
            put(CORE + 'src/compact_receipts.rs', output + ';\n')
            put(CORE + 'build.rs', '// bound build\n')
            put(CORE + 'build_support/compact_receipts.rs', '// bound generator\n')
            self.assertEqual(registration.audit_literal_inputs(root), {'data/anasazi/Map.txt'})
            file = root / (CORE + 'src/compact_receipts.rs')
            for replacement in [output.replace('OUT_DIR', 'OTHER_DIR'), output.replace('compact_raw_receipts', 'forged'), output + ';' + output]:
                file.write_text(replacement)
                with self.assertRaises(ValueError):
                    registration.audit_literal_inputs(root)
            file.write_text(output)
            (root / (CORE + 'build_support/compact_receipts.rs')).unlink()
            with self.assertRaises((ValueError, OSError)):
                registration.audit_literal_inputs(root)

    def test_notes_wrapper_rejects_unresolved_or_commented_calls(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            put = self.fixture(root)
            definition = '''macro_rules! input {
                ($p:literal) => {
                    (concat!("crates/sugarscape-core/src/persistent_notes/", $p), identity_bytes!($p),)
                };
            }'''
            path = CORE + 'src/persistent_notes/source_inputs.rs'
            good = definition + '\ninput!("payload.rs");'
            put(path, good)
            put(CORE + 'src/persistent_notes/payload.rs', '// bound input')
            self.assertEqual(registration.audit_literal_inputs(root), {'data/anasazi/Map.txt'})
            for changed in [good.replace('$p:literal', '$p:expr'),
                            good.replace('input!("payload.rs")', 'input!(concat!("payload.rs"))'),
                            good.replace('input!("payload.rs")', 'input/**/!("payload.rs")')]:
                put(path, changed)
                with self.assertRaises(ValueError):
                    registration.audit_literal_inputs(root)

    def test_selected_generated_assets_reject_unknown_or_missing_output(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp).resolve()
            put = self.fixture(root)
            good = '\n'.join('include_str!(concat!(env!("OUT_DIR"), "/' + name + '-selected.json"));'
                             for name in ['strategic', 'strategy', 'audit'])
            path = CORE + 'src/browser_experiments/recorded.rs'
            put(path, good)
            put(CORE + 'build.rs', '// bound build')
            put(CORE + 'build_support/compact_receipts.rs', '// bound generator')
            self.assertEqual(registration.audit_literal_inputs(root), {'data/anasazi/Map.txt'})
            for changed in [good.replace('audit-selected', 'unknown-selected'),
                            good.replace('OUT_DIR', 'OTHER_DIR'), good + good]:
                put(path, changed)
                with self.assertRaises(ValueError):
                    registration.audit_literal_inputs(root)
