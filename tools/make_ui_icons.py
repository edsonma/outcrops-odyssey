#!/usr/bin/env python3
"""Draw the touch-button icons (assets/ui/icon_*.png). Pure Pillow, antialiased by 4x supersampling.
   python3 tools/make_ui_icons.py"""
from PIL import Image, ImageDraw
import os, math

OUT = os.path.join(os.path.dirname(__file__), "..", "bevy-prototype", "assets", "ui")
os.makedirs(OUT, exist_ok=True)
S = 96          # final size
K = 4           # supersampling
W = S * K
INK = (250, 240, 214, 255)      # warm white glyphs
ACC = (240, 160, 74, 255)       # orange accent
BLUE = (110, 178, 238, 255)
GREEN = (126, 200, 110, 255)
DARK = (34, 28, 20, 255)

def canvas():
    im = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    return im, ImageDraw.Draw(im)

def p(x):  # 0..96 -> supersampled
    return x * K

def poly(d, pts, fill):
    d.polygon([(p(x), p(y)) for x, y in pts], fill=fill)

def rr(d, box, r, fill, outline=None, width=0):
    d.rounded_rectangle([p(box[0]), p(box[1]), p(box[2]), p(box[3])], radius=p(r), fill=fill, outline=outline, width=p(width))

def ell(d, box, fill, outline=None, width=0):
    d.ellipse([p(box[0]), p(box[1]), p(box[2]), p(box[3])], fill=fill, outline=outline, width=p(width))

def line(d, a, b, w, fill):
    d.line([(p(a[0]), p(a[1])), (p(b[0]), p(b[1]))], fill=fill, width=p(w))

def save(im, name):
    im.resize((S, S), Image.LANCZOS).save(os.path.join(OUT, f"icon_{name}.png"))

# --- backpack
im, d = canvas()
rr(d, (30, 10, 66, 24), 7, None, INK, 4)                 # top handle
rr(d, (22, 20, 74, 84), 16, INK)                          # body
rr(d, (30, 52, 66, 76), 7, ACC)                           # pocket
line(d, (30, 62), (66, 62), 3, DARK)
rr(d, (34, 30, 62, 40), 4, DARK)                          # flap strap
save(im, "pack")

# --- truck (pickup, side view)
im, d = canvas()
rr(d, (8, 40, 88, 66), 5, INK)                            # body
poly(d, [(14, 40), (24, 22), (50, 22), (58, 40)], INK)    # cab
poly(d, [(20, 40), (27, 26), (36, 26), (36, 40)], DARK)   # windows
poly(d, [(40, 40), (40, 26), (48, 26), (53, 40)], DARK)
rr(d, (58, 44, 86, 52), 2, ACC)                           # bed stripe
ell(d, (16, 54, 38, 76), DARK); ell(d, (21, 59, 33, 71), INK)
ell(d, (60, 54, 82, 76), DARK); ell(d, (65, 59, 77, 71), INK)
save(im, "truck")

# --- water drop + bite (fill / eat)
im, d = canvas()
poly(d, [(48, 8), (24, 52), (72, 52)], BLUE)
ell(d, (22, 36, 74, 84), BLUE)
ell(d, (34, 52, 44, 66), (255, 255, 255, 140))
save(im, "use")

# --- pickaxe (break)
im, d = canvas()
line(d, (24, 80), (66, 30), 9, (170, 120, 70, 255))       # handle
d.arc([p(14), p(8), p(86), p(58)], 200, 340, fill=INK, width=p(11))  # curved head
poly(d, [(12, 40), (22, 22), (26, 26), (18, 42)], INK)    # head tips
poly(d, [(84, 40), (74, 22), (70, 26), (78, 42)], INK)
save(im, "break")

# --- map (folded) + pin
im, d = canvas()
poly(d, [(8, 24), (32, 16), (32, 76), (8, 84)], INK)
poly(d, [(34, 16), (62, 24), (62, 84), (34, 76)], (225, 214, 184, 255))
poly(d, [(64, 24), (88, 16), (88, 76), (64, 84)], INK)
line(d, (14, 60), (28, 36), 3, GREEN); line(d, (40, 64), (56, 40), 3, BLUE)
ell(d, (50, 14, 68, 32), ACC); poly(d, [(52, 28), (66, 28), (59, 44)], ACC); ell(d, (56, 20, 62, 26), DARK)
save(im, "map")

# --- cards (two overlapping)
im, d = canvas()
back = Image.new("RGBA", (W, W), (0, 0, 0, 0)); bd = ImageDraw.Draw(back)
bd.rounded_rectangle([p(14), p(18), p(54), p(76)], radius=p(7), fill=(218, 206, 176, 255))
back = back.rotate(-14, resample=Image.BICUBIC, center=(p(34), p(60)))
im.alpha_composite(back)
rr(d, (36, 16, 78, 80), 7, INK)
rr(d, (42, 22, 72, 52), 4, ACC)
line(d, (44, 62), (70, 62), 4, DARK); line(d, (44, 71), (62, 71), 4, DARK)
save(im, "cards")

# --- camera view
im, d = canvas()
rr(d, (10, 26, 86, 76), 10, INK)
rr(d, (32, 16, 60, 30), 4, INK)
ell(d, (30, 33, 66, 69), DARK); ell(d, (37, 40, 59, 62), ACC)
save(im, "cam")

# --- close / prev / next
im, d = canvas()
line(d, (26, 26), (70, 70), 11, INK); line(d, (70, 26), (26, 70), 11, INK)
save(im, "close")
im, d = canvas(); poly(d, [(48, 22), (76, 70), (20, 70)], INK); save(im, "prev")
im, d = canvas(); poly(d, [(20, 26), (76, 26), (48, 74)], INK); save(im, "next")
print("icons written to", os.path.abspath(OUT))
