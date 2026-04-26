#!/usr/bin/env python3
"""
npietedit - a simple editor for the Piet programming language
Python+tkinter port of npietedit-0.9d.tcl by Erik Schoenfelder (schoenfr@web.de)
"""

import sys
import os
import collections
import tkinter as tk
from tkinter import messagebox, filedialog

try:
    from PIL import Image as PILImage
    HAS_PIL = True
except ImportError:
    HAS_PIL = False

VERSION = "npietedit v0.9d (py)"

# ---------------------------------------------------------------------------
# Color tables
# ---------------------------------------------------------------------------

# 18 Piet colors (indices 0-17): 6 hues × 3 lightness levels
# Index 18 = black, index 19 = white
COLORS = {
     0: "#FFC0C0",  1: "#FFFFC0",  2: "#C0FFC0",
     3: "#C0FFFF",  4: "#C0C0FF",  5: "#FFC0FF",
     6: "#FF0000",  7: "#FFFF00",  8: "#00FF00",
     9: "#00FFFF", 10: "#0000FF", 11: "#FF00FF",
    12: "#C00000", 13: "#C0C000", 14: "#00C000",
    15: "#00C0C0", 16: "#0000C0", 17: "#C000C0",
}

# RGB components per color index (for PPM export)
C_RED   = {0:255,1:255,2:192,3:192,4:192,5:255,6:255,7:255,8:0,  9:0,  10:0,  11:255,12:192,13:192,14:0,  15:0,  16:0,  17:192,18:0,  19:255}
C_GREEN = {0:192,1:255,2:255,3:255,4:192,5:192,6:0,  7:255,8:255,9:255,10:0,  11:0,  12:0,  13:192,14:192,15:192,16:0,  17:0,  18:0,  19:255}
C_BLUE  = {0:192,1:192,2:192,3:255,4:255,5:255,6:0,  7:0,  8:0,  9:255,10:255,11:255,12:0,  13:0,  14:0,  15:192,16:192,17:192,18:0,  19:255}

# Commands indexed by (lightness_change * 6 + hue_change)
COMMANDS = {
     0: "nop",       6: "push",     12: "pop",
     1: "add",       7: "sub",      13: "mul",
     2: "div",       8: "mod",      14: "not",
     3: "greater",   9: "pointer",  15: "switch",
     4: "dup",      10: "roll",     16: "in(num)",
     5: "in(char)", 11: "out(num)", 17: "out(char)",
}


def idx2col(idx):
    if idx == 18:
        return "#000000"
    if idx == 19:
        return "#ffffff"
    if 0 <= idx < 18:
        return COLORS[idx]
    return "#ffffff"


def col2idx(r, g, b):
    s = "#{:02X}{:02X}{:02X}".format(r, g, b)
    if s == "#000000":
        return 18
    if s == "#FFFFFF":
        return 19
    for i, c in COLORS.items():
        if c.upper() == s:
            return i
    return 19  # unknown → white


# ---------------------------------------------------------------------------
# Drawing helpers
# ---------------------------------------------------------------------------

def paint_rect(canvas, x, y, idx, zpx, zpy=None):
    if zpy is None:
        zpy = zpx
    px = 2 + x * zpx
    py = 2 + y * zpy
    col = idx2col(idx)
    canvas.create_rectangle(px, py, px + zpx - 2, py + zpy - 2, fill=col, outline=col)


def paint_border(canvas, x, y, flag, zpx, zpy=None):
    if zpy is None:
        zpy = zpx
    px = 1 + x * zpx
    py = 1 + y * zpy
    outline = "#000000" if flag else "#ffffff"
    canvas.create_rectangle(px, py, px + zpx, py + zpy, fill="", outline=outline)


# ---------------------------------------------------------------------------
# Application state (module-level, mirroring the original's globals)
# ---------------------------------------------------------------------------

c_maxx = 48
c_maxy = 32
c_zc = 20          # palette zoom (fixed)
c_zpx = 20         # paint canvas x-zoom (adjusts on resize)
c_zpy = 20         # paint canvas y-zoom (adjusts on resize)
c_width  = c_zpx * c_maxx
c_height = c_zpy * c_maxy

cells = {}          # (x, y) -> color index
for _y in range(c_maxy):
    for _x in range(c_maxx):
        cells[(_x, _y)] = 19  # default white

cur_x   = -1
cur_y   = -1
cur_idx = 19        # currently selected color (default white)

cpick_x = -1
cpick_y = -1

filename = "npietedit-filename.ppm"


# ---------------------------------------------------------------------------
# Connected-cell count (iterative BFS — avoids Python recursion limit)
# ---------------------------------------------------------------------------

def count_col_at(x, y):
    global c_maxx, c_maxy
    target = cells.get((x, y))
    if target is None or target >= 18:
        return 0
    visited = set()
    queue = collections.deque([(x, y)])
    visited.add((x, y))
    while queue:
        cx, cy = queue.popleft()
        for nx, ny in ((cx+1, cy), (cx-1, cy), (cx, cy+1), (cx, cy-1)):
            if (nx, ny) in visited:
                continue
            if nx < 0 or nx >= c_maxx or ny < 0 or ny >= c_maxy:
                continue
            if cells.get((nx, ny)) == target:
                visited.add((nx, ny))
                queue.append((nx, ny))
        if len(visited) > 990:
            return "???"
    return len(visited)


def count_col():
    global cur_x, cur_y, cur_idx
    if cur_idx >= 18:
        return 0
    if cells.get((cur_x, cur_y)) != cur_idx:
        return 0
    return count_col_at(cur_x, cur_y)


# ---------------------------------------------------------------------------
# Info display helpers
# ---------------------------------------------------------------------------

def draw_conn(n):
    cpick_canvas.delete("conn_info")
    cpick_canvas.create_rectangle(150, 2, 340, 18, fill="white", outline="grey", tags="conn_info")
    if n and n >= 1:
        label = str(n)
        if isinstance(n, int) and 32 <= n <= 127:
            label = "{} (char: '{}')".format(n, chr(n))
        cpick_canvas.create_text(151, 4, anchor="nw", text="connected: {}".format(label), tags="conn_info")


def draw_pos(x, y):
    cpick_canvas.delete("pos_info")
    cpick_canvas.create_rectangle(150, 22, 340, 38, fill="white", outline="grey", tags="pos_info")
    cpick_canvas.create_text(151, 24, anchor="nw",
        text="px={} py={}, x={} y={}".format(cur_x, cur_y, x, y), tags="pos_info")


# ---------------------------------------------------------------------------
# Paint canvas event handlers
# ---------------------------------------------------------------------------

def click_canvas(event, button):
    global cur_x, cur_y, cur_idx
    x = event.x // c_zpx
    y = event.y // c_zpy
    if x < 0 or y < 0 or x >= c_maxx or y >= c_maxy:
        return

    if button != 1:
        # right/middle click: pick color from cell
        cur_idx = cells.get((x, y), 19)
        pick_color(cur_idx)
    else:
        # left click: paint cell
        cells[(x, y)] = cur_idx
        paint_rect(paint_canvas, x, y, cur_idx, c_zpx, c_zpy)

    if cur_x >= 0 and cur_y >= 0:
        paint_border(paint_canvas, cur_x, cur_y, 0, c_zpx, c_zpy)
    paint_border(paint_canvas, x, y, 1, c_zpx, c_zpy)

    cur_x = x
    cur_y = y

    draw_conn(count_col())
    draw_pos(x, y)


def motion_canvas(event):
    x = event.x // c_zpx
    y = event.y // c_zpy
    if x < 0 or y < 0 or x >= c_maxx or y >= c_maxy:
        return
    draw_pos(x, y)
    draw_conn(count_col_at(x, y))


# ---------------------------------------------------------------------------
# Palette (color picker) event handler
# ---------------------------------------------------------------------------

def pick_color(idx):
    global cpick_x, cpick_y
    if idx == 19:
        x, y = 6, 1
    elif idx == 18:
        x, y = 6, 0
    else:
        x, y = idx % 6, idx // 6

    if cpick_x >= 0 and cpick_y >= 0:
        paint_border(cpick_canvas, cpick_x, cpick_y, 0, c_zc)
    paint_border(cpick_canvas, x, y, 1, c_zc)
    cpick_x = x
    cpick_y = y


def cpick(event):
    global cur_idx
    x = event.x // c_zc
    y = event.y // c_zc
    if x == 6 and y == 0:
        n_idx = 18
    elif x == 6 and y == 1:
        n_idx = 19
    elif 0 <= x <= 5 and 0 <= y <= 2:
        n_idx = y * 6 + x
    else:
        return
    cur_idx = n_idx
    pick_color(cur_idx)
    redraw_cmd_canvas()


# ---------------------------------------------------------------------------
# Command canvas event handler
# ---------------------------------------------------------------------------

def cmd_click(event):
    global cur_idx
    x = event.x // 56
    y = event.y // c_zc
    n_idx = x + y * 6
    if n_idx < 18 and x < 6 and y < 3:
        cmd_hue   = n_idx % 6
        cmd_light = ((n_idx // 6) + 3) % 3
        ncx = ((cur_idx % 6) + cmd_hue)   % 6
        ncy = ((cur_idx // 6) + cmd_light) % 3
        cur_idx = ncx + 6 * ncy
        pick_color(cur_idx)
        redraw_cmd_canvas()


def redraw_cmd_canvas():
    cmd_canvas.delete("all")
    for i in range(18):
        cx = 2 + 56 * (i % 6)
        cy = 2 + c_zc * (i // 6)
        cmd_hue   = i % 6
        cmd_light = (i // 6) % 3
        ncx = ((cur_idx % 6) + cmd_hue)   % 6
        ncy = ((cur_idx // 6) + cmd_light) % 3
        result_idx = ncx + 6 * ncy
        fill_col = idx2col(result_idx)
        text_col = "white" if result_idx in (10, 16) else "black"
        cmd_canvas.create_rectangle(cx, cy, cx + 54, cy + c_zc - 2,
                                    fill=fill_col, outline="black")
        cmd_canvas.create_text(cx + 2, cy + 2, anchor="nw", text=COMMANDS[i], fill=text_col)


# ---------------------------------------------------------------------------
# File I/O
# ---------------------------------------------------------------------------

def save_cells(fname):
    try:
        with open(fname, "w") as fp:
            fp.write("P3\n# a piet program {}\n{} {}\n255\n".format(fname, c_maxx, c_maxy))
            n = 0
            for y in range(c_maxy):
                for x in range(c_maxx):
                    idx = cells.get((x, y), 19)
                    fp.write("{:3d} {:3d} {:3d}  ".format(C_RED[idx], C_GREEN[idx], C_BLUE[idx]))
                    n += 1
                    if n % 6 == 0:
                        fp.write("\n")
            fp.write("\n")
    except OSError as e:
        messagebox.showerror("Save error", str(e))


def load_p3_cells(fname):
    """Parse a P3 ASCII PPM file. Returns (width, height) on success or None."""
    try:
        with open(fname, "r") as fp:
            content = fp.read()
    except (OSError, UnicodeDecodeError, ValueError):
        return None

    tokens = []
    for line in content.splitlines():
        stripped = line.strip()
        if stripped.startswith("#"):
            continue
        tokens.extend(stripped.split())

    if not tokens or tokens[0] != "P3":
        return None

    try:
        width  = int(tokens[1])
        height = int(tokens[2])
        # tokens[3] is max color value (255 expected)
        rgb_tokens = tokens[4:]
        for y in range(height):
            for x in range(width):
                base = (y * width + x) * 3
                r, g, b = int(rgb_tokens[base]), int(rgb_tokens[base+1]), int(rgb_tokens[base+2])
                cells[(x, y)] = col2idx(r, g, b)
    except (IndexError, ValueError):
        return None

    return width, height


def load_cells(fname):
    global c_maxx, c_maxy
    result = load_p3_cells(fname)
    if result:
        cells_resize(result[0], result[1], recalc_zoom=True)
        return

    if HAS_PIL:
        try:
            img = PILImage.open(fname).convert("RGB")
            w, h = img.size
            for y in range(h):
                for x in range(w):
                    r, g, b = img.getpixel((x, y))
                    cells[(x, y)] = col2idx(r, g, b)
            cells_resize(w, h, recalc_zoom=True)
            return
        except Exception as e:
            messagebox.showerror("Load error", "Cannot read {}: {}".format(fname, e))
            return

    messagebox.showerror("Load error",
        "Cannot read {}.\nInstall Pillow (pip install Pillow) for GIF and non-P3 PPM support.".format(fname))


def on_save():
    global filename
    fname = filedialog.asksaveasfilename(
        initialfile=filename,
        filetypes=[("PPM files", "*.ppm"), ("All files", "*")],
        defaultextension=".ppm",
    ) or filename
    filename = fname
    save_cells(fname)


def on_load():
    global filename
    fname = filedialog.askopenfilename(
        initialfile=filename,
        filetypes=[("Piet files", "*.ppm *.gif"), ("PPM files", "*.ppm"), ("GIF files", "*.gif"), ("All files", "*")],
    )
    if fname:
        filename = fname
        load_cells(fname)


# ---------------------------------------------------------------------------
# Grid resize
# ---------------------------------------------------------------------------

def cells_enlarge(delta):
    if delta < 0 and (c_maxx <= 4 or c_maxy <= 4):
        return
    cells_resize(c_maxx + delta, c_maxy + delta)


def cells_enlarge_x(delta):
    if delta < 0 and c_maxx <= 4:
        return
    cells_resize(c_maxx + delta, c_maxy)


def cells_enlarge_y(delta):
    if delta < 0 and c_maxy <= 4:
        return
    cells_resize(c_maxx, c_maxy + delta)


def cells_resize(width, height, recalc_zoom=False):
    global c_maxx, c_maxy, c_zpx, c_zpy, c_width, c_height
    c_maxx = width
    c_maxy = height

    if recalc_zoom:
        _MAX = 960
        c_zpx = max(8, min(20, _MAX // c_maxx)) if c_maxx > 0 else 20
        c_zpy = max(8, min(20, _MAX // c_maxy)) if c_maxy > 0 else 20

    c_width  = c_zpx * c_maxx
    c_height = c_zpy * c_maxy
    paint_canvas.config(width=c_width + 1, height=c_height + 1)

    paint_canvas.create_rectangle(0, 0, c_width + 1, c_height + 1, fill="white", outline="")
    for y in range(c_maxy):
        for x in range(c_maxx):
            idx = cells.get((x, y), 19)
            cells[(x, y)] = idx
            paint_rect(paint_canvas, x, y, idx, c_zpx, c_zpy)


# ---------------------------------------------------------------------------
# Build the UI
# ---------------------------------------------------------------------------

root = tk.Tk()
root.title("npietedit")

# -- Toolbar -----------------------------------------------------------------
toolbar = tk.Frame(root, borderwidth=10)
toolbar.pack(side="top", fill="x")

tk.Label(toolbar, text=VERSION).pack(side="left")
tk.Button(toolbar, text="Quit",    command=root.quit).pack(side="left")
tk.Button(toolbar, text="Save",    command=on_save).pack(side="left")
tk.Button(toolbar, text="Load",    command=on_load).pack(side="left")
tk.Button(toolbar, text="Shrink",  command=lambda: cells_enlarge(-5)).pack(side="left")
tk.Button(toolbar, text="Enlarge", command=lambda: cells_enlarge(+5)).pack(side="left")
tk.Button(toolbar, text="Wider",   command=lambda: cells_enlarge_x(+5)).pack(side="left")
tk.Button(toolbar, text="Narrower",command=lambda: cells_enlarge_x(-5)).pack(side="left")
tk.Button(toolbar, text="Taller",  command=lambda: cells_enlarge_y(+5)).pack(side="left")
tk.Button(toolbar, text="Shorter", command=lambda: cells_enlarge_y(-5)).pack(side="left")

# -- Color picker canvas -----------------------------------------------------
cpick_frame = tk.Frame(root)
cpick_frame.pack(side="top", fill="x")

cpick_canvas = tk.Canvas(cpick_frame, bg="white",
    width=c_width, height=c_zc * 3 + 2)
cpick_canvas.pack(fill="both")

# Draw palette
for i in range(18):
    paint_rect(cpick_canvas, i % 6, i // 6, i, c_zc)
paint_rect(cpick_canvas, 6, 0, 18, c_zc)  # black
paint_rect(cpick_canvas, 6, 1, 19, c_zc)  # white

# Highlight default (white)
paint_border(cpick_canvas, 6, 1, 1, c_zc)
cpick_x = 6
cpick_y = 1

cpick_canvas.bind("<Button-1>", cpick)

# -- Command reference canvas ------------------------------------------------
cmd_frame = tk.Frame(root)
cmd_frame.pack(side="top", fill="x")

cmd_canvas = tk.Canvas(cmd_frame, bg="white",
    width=c_width, height=c_zc * 3 + 2)
cmd_canvas.pack(fill="both")

redraw_cmd_canvas()

cmd_canvas.bind("<Button-1>", cmd_click)

# -- Paint canvas ------------------------------------------------------------
paint_frame = tk.Frame(root)
paint_frame.pack(side="left", fill="both")

paint_canvas = tk.Canvas(paint_frame, bg="white",
    width=c_width + 1, height=c_height + 1)
paint_canvas.create_rectangle(0, 0, c_width + 1, c_height + 1, fill="white", outline="")
paint_canvas.pack(fill="both")

# Initial paint
for _y in range(c_maxy):
    for _x in range(c_maxx):
        paint_rect(paint_canvas, _x, _y, 19, c_zpx, c_zpy)

paint_canvas.bind("<Motion>",          motion_canvas)
paint_canvas.bind("<Button-1>",        lambda e: click_canvas(e, 1))
paint_canvas.bind("<Button-2>",        lambda e: click_canvas(e, 2))  # middle (X11 / Mac)
paint_canvas.bind("<Button-3>",        lambda e: click_canvas(e, 3))  # right (Windows)
paint_canvas.bind("<B1-Motion>",       lambda e: click_canvas(e, 1))

# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    if len(sys.argv) == 2:
        arg = sys.argv[1]
        if arg.startswith("-"):
            print("\n{}\n".format(VERSION))
            print("use: npietedit [<filename>]")
            sys.exit(0)
        filename = arg
        print("filename:", filename)
        if os.path.isfile(filename):
            load_cells(filename)
    elif len(sys.argv) > 2:
        print("\n{}\n".format(VERSION))
        print("use: npietedit [<filename>]")
        sys.exit(0)

    root.mainloop()
