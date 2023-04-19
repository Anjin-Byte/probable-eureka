import math
    
class Hex:
    def __init__(self, q, r, s):
        assert not (round(q + r + s) != 0), "q + r + s must be 0"
        self.q = q
        self.r = r
        self.s = s


    @staticmethod
    def equal(a, b):
        return a.q == b.q and a.r == b.r and a.s == b.s;


    @staticmethod
    def hex_add(a, b):
        return Hex(a.q + b.q, a.r + b.r, a.s + b.s)


    @staticmethod
    def hex_subtract(a, b):
        return Hex(a.q - b.q, a.r - b.r, a.s - b.s)


    @staticmethod
    def hex_scale(a, k):
        return Hex(a.q * k, a.r * k, a.s * k)


    @staticmethod
    def hex_rotate_left(a):
        return Hex(-a.s, -a.q, -a.r)


    @staticmethod
    def hex_rotate_right(a):
        return Hex(-a.r, -a.s, -a.q)
    

    @staticmethod
    def hex_direction(direction):
        hex_directions = [Hex(1, 0, -1), Hex(1, -1, 0), 
                            Hex(0, -1, 1), Hex(-1, 0, 1), 
                            Hex(-1, 1, 0), Hex(0, 1, -1)]
        return hex_directions[direction]
    

    @staticmethod
    def hex_neighbor(hex, direction):
        return Hex.hex_add(hex, Hex.hex_direction(direction))

    
    @staticmethod
    def hex_diagonal_direction(direction):
        hex_diagonals = [Hex(2, -1, -1), Hex(1, -2, 1), 
                            Hex(-1, -1, 2), Hex(-2, 1, 1), 
                            Hex(-1, 2, -1), Hex(1, 1, -2)]
        return hex_diagonals[direction]


    @staticmethod
    def hex_diagonal_neighbor(hex, direction):
        return Hex.add(hex, Hex.hex_diagonal_direction[direction])


    @staticmethod
    def hex_length(a):
        return (abs(a.q) + abs(a.r) + abs(a.s)) // 2


    @staticmethod
    def hex_distance(a, b):
        return Hex.hex_length(Hex.hex_subtract(a, b))


    @staticmethod
    def hex_round(a):
        qi = int(round(a.q))
        ri = int(round(a.r))
        si = int(round(a.s))
        q_diff = abs(qi - a.q)
        r_diff = abs(ri - a.r)
        s_diff = abs(si - a.s)
        if q_diff > r_diff and q_diff > s_diff:
            qi = -ri - si
        else:
            if r_diff > s_diff:
                ri = -qi - si
            else:
                si = -qi - ri
        return Hex(qi, ri, si)

    @staticmethod
    def hex_lerp(a, b, t):
        return Hex(a.q * (1.0 - t) + b.q * t, a.r * (1.0 - t) + b.r * t, a.s * (1.0 - t) + b.s * t)

    @staticmethod
    def hex_linedraw(a, b):
        N = Hex.hex_distance(a, b)
        a_nudge = Hex(a.q + 1e-06, a.r + 1e-06, a.s - 2e-06)
        b_nudge = Hex(b.q + 1e-06, b.r + 1e-06, b.s - 2e-06)
        results = []
        step = 1.0 / max(N, 1)
        for i in range(0, N + 1):
            results.append(Hex.hex_round(Hex.hex_lerp(a_nudge, b_nudge, step * i)))
        return results


    EVEN = 1
    ODD = -1
    @staticmethod
    def qoffset_from_cube(offset, a):
        col = a.q
        row = a.r + (a.q + offset * (a.q & 1)) // 2
        if offset != Hex.EVEN and offset != Hex.ODD:
            raise ValueError("offset must be EVEN (+1) or ODD (-1)")
        return [col, row]


    @staticmethod
    def qoffset_to_cube(offset, col, row):
        q = col
        r = row - (col + offset * (col & 1)) // 2
        s = -q - r
        if offset != Hex.EVEN and offset != Hex.ODD:
            raise ValueError("offset must be EVEN (+1) or ODD (-1)")
        return Hex(q, r, s)


    @staticmethod
    def roffset_from_cube(offset, a):
        col = a.q + (a.r + offset * (a.r & 1)) // 2
        row = a.r
        if offset != Hex.EVEN and offset != Hex.ODD:
            raise ValueError("offset must be EVEN (+1) or ODD (-1)")
        return [col, row]


    @staticmethod
    def roffset_to_cube(offset, col, row):
        q = col - (row + offset * (row & 1)) // 2
        r = row
        s = -q - r
        if offset != Hex.EVEN and offset != Hex.ODD:
            raise ValueError("offset must be EVEN (+1) or ODD (-1)")
        return Hex(q, r, s)


    @staticmethod
    def qdoubled_from_cube(a):
        col = a.q
        row = 2 * a.r + a.q
        return [col, row]


    @staticmethod
    def qdoubled_to_cube(a):
        q = a.col
        r = (a.row - a.col) // 2
        s = -q - r
        return Hex(q, r, s)


    @staticmethod
    def rdoubled_from_cube(a):
        col = 2 * a.q + a.r
        row = a.r
        return [col, row]


    @staticmethod
    def rdoubled_to_cube(a):
        q = (a.col - a.row) // 2
        r = a.row
        s = -q - r
        return Hex(q, r, s)

class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

class Layout:
    class Orientation:
        def __init__(self, state):
            self.f0 = state[0]
            self.f1 = state[1]
            self.f2 = state[2]
            self.f3 = state[3]
            self.b0 = state[4]
            self.b1 = state[5]
            self.b2 = state[6]
            self.b3 = state[7]
            self.start_angle = state[8]

    pointy = [math.sqrt(3.0), math.sqrt(3.0) / 2.0, 
        0.0, 3.0 / 2.0, math.sqrt(3.0) / 3.0, -1.0 / 3.0, 0.0, 2.0 / 3.0, 0.5]

    flat = [3.0 / 2.0, 0.0, math.sqrt(3.0) / 2.0, 
        math.sqrt(3.0), 2.0 / 3.0, 0.0, -1.0 / 3.0, math.sqrt(3.0) / 3.0, 0.0]

    def __init__(self, orientation, size, origin):
        if orientation == "pointy":
            self.orientation = Layout.Orientation(Layout.pointy)
        if orientation == "flat":
            self.orientation = Layout.Orientation(Layout.flat)
        self.size = size;
        self.origin = origin;

    @staticmethod
    def hex_to_pixel(layout, h):
        M = layout.orientation
        size = layout.size
        origin = layout.origin
        x = (M.f0 * h.q + M.f1 * h.r) * size.x
        y = (M.f2 * h.q + M.f3 * h.r) * size.y
        return Point(x + origin.x, y + origin.y)

    @staticmethod
    def pixel_to_hex(layout, p):
        M = layout.orientation
        size = layout.size
        origin = layout.origin
        pt = Point((p.x - origin.x) / size.x, (p.y - origin.y) / size.y)
        q = M.b0 * pt.x + M.b1 * pt.y
        r = M.b2 * pt.x + M.b3 * pt.y
        return Hex(q, r, -q - r)

    @staticmethod
    def hex_corner_offset(layout, corner):
        M = layout.orientation
        size = layout.size
        angle = 2.0 * math.pi * (M.start_angle - corner) / 6.0
        return Point(size.x * math.cos(angle), size.y * math.sin(angle))

    @staticmethod
    def polygon_corners(layout, h):
        corners = []
        center = Layout.hex_to_pixel(layout, h)
        for i in range(0, 6):
            offset = Layout.hex_corner_offset(layout, i)
            corners.append(Point(center.x + offset.x, center.y + offset.y))
        return corners

