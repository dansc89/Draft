import importlib.util
from pathlib import Path
import tempfile
import unittest
import ezdxf
from pypdf import PdfWriter
from pypdf.generic import DecodedStreamObject, NameObject

spec = importlib.util.spec_from_file_location('verify_demo', Path(__file__).with_name('verify-demo.py'))
assert spec is not None and spec.loader is not None
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class IndependentCheckerTests(unittest.TestCase):
    def fixture(self, directory, operator):
        doc = ezdxf.new('R2010')
        doc.units = 1
        for a,b in [((0,0),(144,0)), ((144,0),(144,96)), ((144,96),(0,96)), ((0,96),(0,0))]:
            doc.modelspace().add_line(a,b)
        doc.saveas(directory/'demo.dxf')
        writer = PdfWriter()
        page = writer.add_blank_page(width=792, height=612)
        stream = DecodedStreamObject()
        stream.set_data(b'36 36 m 252 36 l 252 180 l 36 180 l h '+operator+b'\n')
        page[NameObject('/Contents')] = writer._add_object(stream)
        writer.write(directory/'demo.pdf')

    def test_painted_fixture_passes(self):
        with tempfile.TemporaryDirectory() as d:
            directory = Path(d)
            self.fixture(directory, b'S')
            checker.verify(directory)

    def test_unpainted_geometry_is_not_a_successful_vector_export(self):
        with tempfile.TemporaryDirectory() as d:
            directory = Path(d)
            self.fixture(directory, b'n')
            with self.assertRaises(AssertionError):
                checker.verify(directory)


if __name__ == '__main__':
    unittest.main()
