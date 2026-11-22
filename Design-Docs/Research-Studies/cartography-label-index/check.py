"""#68 isolated label collision broadphase; synthetic premeasured rectangles.
Not renderer, geometry persistence, policy or native product acceptance evidence.
"""
from collections import defaultdict
from math import floor
from random import Random
from time import perf_counter

WIDTH, HEIGHT, CELL = 1024, 768, 64


def intersects(a, b):
    return a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3]


def cells(box):
    x0, y0, x1, y1 = box
    # Conservative closed cell coverage; exact intersection rejects touching edges.
    return ((x, y) for x in range(floor(x0 / CELL), floor(x1 / CELL) + 1)
            for y in range(floor(y0 / CELL), floor(y1 / CELL) + 1))


def layout(records, indexed):
    placed, grid, comparisons = [], defaultdict(list), 0
    unknown = []
    # Supplied visibility is a fixture stand-in for the upstream policy, not policy itself.
    for ident, priority, anchor, w, h, visible in sorted(
            records, key=lambda r: (-r[1], r[0])):
        if not visible:
            continue
        if anchor is None:
            unknown.append(ident)
            continue
        x, y = anchor
        for left, top in ((x + 6, y - h - 4), (x + 6, y + 4),
                          (x - w - 6, y - h - 4), (x - w - 6, y + 4)):
            box = left, top, left + w, top + h
            if box[0] < 0 or box[1] < 0 or box[2] > WIDTH or box[3] > HEIGHT:
                continue
            candidates = (sorted({i for c in cells(box) for i in grid.get(c, ())})
                          if indexed else range(len(placed)))
            collision = False
            for i in candidates:
                comparisons += 1
                if intersects(box, placed[i][1]):
                    collision = True
                    break
            if not collision:
                if indexed:
                    for c in cells(box):
                        grid[c].append(len(placed))
                placed.append((ident, box))
                break
    return placed, unknown, comparisons


def fixture(n, seed):
    rng = Random(seed)
    return [(i, rng.randrange(4), (rng.randrange(WIDTH), rng.randrange(HEIGHT)),
             rng.randrange(35, 155), rng.randrange(12, 25), True) for i in range(n)]


def verify(n, seed):
    data = fixture(n, seed)
    start = perf_counter()
    a, u, naive = layout(data, False)
    baseline_ms = (perf_counter() - start) * 1000
    start = perf_counter()
    b, v, indexed = layout(data, True)
    indexed_ms = (perf_counter() - start) * 1000
    assert (a, u) == (b, v)
    assert all(not intersects(x[1], y[1]) for i, x in enumerate(b) for y in b[i + 1:])
    assert layout(list(reversed(data)), True)[:2] == (b, v)
    # Secret feature outranks every visible label and would otherwise displace one.
    secret = (n, 100, (100, 100), 100, 20, False)
    unknown = (n + 1, 100, None, 100, 20, True)
    assert layout(data + [secret, unknown], True)[:2] == (b, [n + 1])
    print(f'{n:5d} {seed:4d} {len(a):4d} {naive:10d} {indexed:10d} '
          f'{baseline_ms:9.2f} {indexed_ms:9.2f}')


if __name__ == '__main__':
    # Touching edges are legal; cell boundaries cannot conceal overlap.
    assert not intersects((0, 0, 64, 64), (64, 0, 100, 64))
    assert intersects((0, 0, 65, 64), (64, 0, 100, 64))
    print('count seed kept naive_tests grid_tests naive_ms grid_ms')
    for size in (1000, 10000, 50000):
        verify(size, 68)
    for seed in range(5):
        verify(1000, seed)
    print('All parity, collision, permutation, hidden and unknown assertions passed')
