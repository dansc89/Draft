#!/usr/bin/env python3
"""Independently verify Draft's generated 12' x 8' DXF/PDF rectangle."""
from pathlib import Path
import math
import sys
import ezdxf
from pypdf import PdfReader
from pypdf.generic import ContentStream


def near(a, b):
    return math.isclose(float(a), float(b), rel_tol=0, abs_tol=1e-6)


def verify(directory):
    directory = Path(directory)
    dxfs, pdfs = list(directory.glob('*.dxf')), list(directory.glob('*.pdf'))
    assert len(dxfs) == len(pdfs) == 1, 'Expected one demo DXF and one demo PDF'
    doc = ezdxf.readfile(dxfs[0])
    assert not doc.audit().has_errors, 'DXF audit failed'
    assert doc.header.get('$INSUNITS') == 1, 'DXF must declare inches'
    entities = list(doc.modelspace())
    assert len(entities) == 4 and all(e.dxftype() == 'LINE' for e in entities)
    def edge(a, b):
        return frozenset((tuple(round(float(v), 6) for v in a[:2]),
                          tuple(round(float(v), 6) for v in b[:2])))
    expected = {edge((0, 0), (144, 0)), edge((144, 0), (144, 96)),
                edge((144, 96), (0, 96)), edge((0, 96), (0, 0))}
    actual = {edge(tuple(e.dxf.start), tuple(e.dxf.end)) for e in entities}
    assert actual == expected, 'DXF full-size endpoints changed'
    reader = PdfReader(pdfs[0], strict=True)
    assert len(reader.pages) == 1
    page = reader.pages[0]
    assert near(page.mediabox.width, 792) and near(page.mediabox.height, 612), 'Landscape Letter required'
    assert page.get('/UserUnit', 1) == 1 and page.get('/Rotate', 0) == 0
    resources = page.get('/Resources', {})
    assert not resources.get('/XObject'), 'Drawing must be vector, not a raster image'
    matrix = (1, 0, 0, 1, 0, 0)
    stack, segments, path, start, last = [], [], [], None, None
    def transform(x, y):
        a, b, c, d, e, f = matrix
        return (a*x+c*y+e, b*x+d*y+f)
    for operands, op in ContentStream(page.get_contents(), reader).operations:
        if op == b'q':
            stack.append(matrix)
        elif op == b'Q':
            matrix = stack.pop()
        elif op == b'cm':
            a,b,c,d,e,f = matrix
            A,B,C,D,E,F = map(float, operands)
            matrix = (a*A+c*B,b*A+d*B,a*C+c*D,b*C+d*D,a*E+c*F+e,b*E+d*F+f)
        elif op == b'm':
            start = last = transform(*map(float, operands))
        elif op == b'l':
            point = transform(*map(float, operands)); path.append((last, point)); last = point
        elif op == b'h':
            if last != start:
                path.append((last, start))
            last = start
        elif op == b're':
            x,y,w,h = map(float, operands)
            p = [transform(x,y), transform(x+w,y), transform(x+w,y+h), transform(x,y+h)]
            path.extend(zip(p, p[1:]+p[:1]))
            start = last = p[0]
        elif op in (b'S', b's', b'B', b'B*', b'b', b'b*'):
            if op in (b's', b'b', b'b*') and last != start:
                path.append((last, start))
            segments.extend(path)
            path = []
        elif op in (b'n', b'f', b'F', b'f*'):
            path = []
    assert len(segments) == 4, 'Expected four vector rectangle edges'
    points = [p for s in segments for p in s]
    min_x, max_x = min(p[0] for p in points), max(p[0] for p in points)
    min_y, max_y = min(p[1] for p in points), max(p[1] for p in points)
    # Physical model inches / 48 * 72 PDF points per paper inch.
    assert near(max_x-min_x, 144/48*72) and near(max_y-min_y, 96/48*72), 'Wrong physical PDF scale'
    assert 0 <= min_x < max_x <= 792 and 0 <= min_y < max_y <= 612, 'Clipped drawing'
    want = {edge((min_x,min_y),(max_x,min_y)), edge((max_x,min_y),(max_x,max_y)),
            edge((max_x,max_y),(min_x,max_y)), edge((min_x,max_y),(min_x,min_y))}
    assert {edge(a,b) for a,b in segments} == want, 'PDF is not the expected rectangle'
    print('PASS: independently audited inch-based DXF; 12-foot by 8-foot geometry; vector PDF at 1:48 (3 by 2 paper inches).')


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('Usage: verify-demo.py NEW_DEMO_DIRECTORY')
    verify(sys.argv[1])
