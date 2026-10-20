#!/usr/bin/env python3
"""Generate tileable procedural PBR textures (albedo .jpg + normal .png) for the 3D scenery.

Needs: numpy, pillow, scipy.   usage: python3 -I tools/make_procedural_textures.py <output dir>

Writes (in <output dir>):  wood, shingles, canvas, asphalt, gravel, crop, bark, leaves,
truck_paint, truck_yellow, wicker, strata  (each *_albedo.jpg + *_normal.png)  and fabric_normal.png.

Everything is periodic (FFT noise / wrapped Voronoi), so it tiles seamlessly. These replace
photo scans where none are available; swap any pair for a real scan with the same file names.
"""
import os
import sys

import numpy as np
from PIL import Image
from scipy.spatial import cKDTree

OUT = sys.argv[1]
os.makedirs(OUT, exist_ok=True)
RNG = np.random.default_rng(20261020)


def norm01(a):
    return (a - a.min()) / (a.max() - a.min() + 1e-9)


def pnoise(n, sx, sy=None, rng=RNG):
    """Periodic smooth noise in 0..1 with correlation length ~sx (x) and ~sy (y) pixels."""
    sy = sx if sy is None else sy
    white = rng.standard_normal((n, n))
    fy = np.fft.fftfreq(n)[:, None]
    fx = np.fft.fftfreq(n)[None, :]
    filt = np.exp(-(((fx * sx) ** 2 + (fy * sy) ** 2) * np.pi ** 2 * 2.0))
    return norm01(np.fft.ifft2(np.fft.fft2(white) * filt).real)


def fbm(n, base, octaves=4, rng=RNG):
    out, amp, tot = np.zeros((n, n)), 1.0, 0.0
    for o in range(octaves):
        out += amp * pnoise(n, max(base / (2 ** o), 1.0), rng=rng)
        tot += amp
        amp *= 0.5
    return norm01(out / tot)


def voronoi(n, points, rng=RNG):
    """Periodic Voronoi: returns (distance to nearest site, id of nearest site) per pixel."""
    pts = rng.uniform(0, n, size=(points, 2))
    tiled = np.concatenate([pts + np.array([dx * n, dy * n]) for dx in (-1, 0, 1) for dy in (-1, 0, 1)])
    ids = np.tile(np.arange(points), 9)
    yy, xx = np.mgrid[0:n, 0:n]
    d, i = cKDTree(tiled).query(np.stack([xx.ravel(), yy.ravel()], 1))
    return d.reshape(n, n), ids[i].reshape(n, n)


def normal_from_height(h, strength):
    dx = (np.roll(h, -1, 1) - np.roll(h, 1, 1)) * 0.5
    dy = (np.roll(h, -1, 0) - np.roll(h, 1, 0)) * 0.5
    n = np.stack([-dx * strength, dy * strength, np.ones_like(h)], -1)
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    return ((n * 0.5 + 0.5) * 255).astype(np.uint8)


def colorize(t, stops):
    """Map 0..1 values through colour stops [(pos, (r,g,b)), ...] -> float RGB 0..255."""
    pos = np.array([p for p, _ in stops])
    cols = np.array([c for _, c in stops], float)
    return np.stack([np.interp(t, pos, cols[:, k]) for k in range(3)], -1)


def save(name, rgb, height, strength, normal_size=None):
    rgb = np.clip(rgb, 0, 255).astype(np.uint8)
    Image.fromarray(rgb).save(os.path.join(OUT, f"{name}_albedo.jpg"), quality=84, optimize=True)
    nrm = normal_from_height(height, strength)
    img = Image.fromarray(nrm)
    if normal_size and normal_size != nrm.shape[0]:
        img = img.resize((normal_size, normal_size), Image.LANCZOS)
    img.save(os.path.join(OUT, f"{name}_normal.png"), optimize=True)
    print("wrote", name, rgb.shape[0], "px")


def planks(n, count, palette, vertical=False, paint=None):
    """Board texture: per-board colour, grain, seams. Returns (rgb, height)."""
    h = n // count
    yy = np.arange(n)[:, None] * np.ones((1, n))
    board = (yy // h).astype(int)
    tone = RNG.uniform(0.82, 1.12, count)[board]
    base = np.array(palette)[RNG.integers(0, len(palette), count)][board] * tone[..., None]
    grain = pnoise(n, 90, 2.2) * 0.6 + pnoise(n, 25, 1.2) * 0.4
    rings = np.sin((pnoise(n, 120, 6) * 9 + np.arange(n)[None, :] / 70.0) * 2 * np.pi) * 0.5 + 0.5
    rgb = base * (0.80 + 0.20 * grain[..., None] + 0.10 * rings[..., None])
    seam = np.exp(-(((yy % h) - 1.0) ** 2) / 3.0) + np.exp(-(((yy % h) - (h - 1.5)) ** 2) / 3.0)
    rgb *= (1.0 - 0.55 * np.clip(seam, 0, 1))[..., None]
    height = grain * 0.35 + rings * 0.15 - seam * 0.9
    if paint is not None:  # chipped paint over wood
        chip = pnoise(n, 20) * 0.6 + pnoise(n, 6) * 0.4
        mask = (chip > 0.62)[..., None]
        coat = np.array(paint) * (0.88 + 0.2 * pnoise(n, 40)[..., None])
        coat = coat * (1.0 - 0.45 * np.clip(seam, 0, 1))[..., None]
        rgb = np.where(mask, rgb * 0.8, coat)
        height = np.where(mask[..., 0], height - 0.25, height)
    if vertical:
        rgb, height = rgb.transpose(1, 0, 2), height.T
    return rgb, height


# --------------------------------------------------------------------------- textures
def make_wood():
    rgb, h = planks(512, 8, [(122, 82, 50), (104, 70, 42), (136, 94, 58), (96, 64, 40)])
    save("wood", rgb, h, 3.0, 512)


def make_shingles():
    """Terracotta barrel tiles: vertical half-round ribs, overlapping courses every 128 px."""
    n = 512
    yy, xx = np.mgrid[0:n, 0:n]
    rib = 0.5 + 0.5 * np.sin(2 * np.pi * (xx + 8) / 32.0)          # 0 in the valley .. 1 on the crown
    course = (yy % 128) / 128.0                                       # 0 top of a tile .. 1 lower lip
    lip = np.exp(-((course - 0.985) ** 2) / 0.00025) + np.exp(-((course - 0.0) ** 2) / 0.0002)
    tone = RNG.uniform(0.82, 1.14, (4, 16))[(yy // 128) % 4, (xx // 32) % 16]
    base = np.array([176, 84, 54]) * tone[..., None]
    shade = 0.55 + 0.45 * rib * (0.75 + 0.25 * (1.0 - course))
    rgb = base * shade[..., None] * (0.88 + 0.22 * fbm(n, 22)[..., None])
    rgb *= (1.0 - 0.55 * np.clip(lip, 0, 1))[..., None]
    moss = np.clip((fbm(n, 30) - 0.62) * 3.0, 0, 1)[..., None]
    rgb = rgb * (1 - 0.35 * moss) + np.array([60, 80, 40]) * 0.35 * moss
    h = rib * 0.8 + (1.0 - course) * 0.25 - np.clip(lip, 0, 1) * 0.5
    save("shingles", rgb, h, 3.0, 512)


def make_canvas():
    n = 256
    yy, xx = np.mgrid[0:n, 0:n]
    wx = np.sin(2 * np.pi * xx / 4.0)
    wy = np.sin(2 * np.pi * yy / 4.0)
    weave = 0.5 + 0.25 * wx * (yy // 2 % 2 * 2 - 1) + 0.25 * wy * (xx // 2 % 2 * 2 - 1)
    stain = fbm(n, 40)
    rgb = np.array([222, 190, 104]) * (0.78 + 0.22 * weave[..., None]) * (0.86 + 0.2 * stain[..., None])
    seam = np.exp(-(((xx % 64) - 1.0) ** 2) / 1.5)  # panel seams every 64 px
    rgb *= (1.0 - 0.35 * seam)[..., None]
    save("canvas", rgb, weave * 0.6 + stain * 0.2, 2.0, 256)


def make_asphalt():
    n = 512
    fine = RNG.random((n, n))
    grit = pnoise(n, 1.2) * 0.5 + pnoise(n, 3.0) * 0.5
    big = fbm(n, 60)
    base = 54 + 22 * grit + 14 * big
    speck = (fine > 0.985) * RNG.uniform(40, 90, (n, n))
    dark = (fine < 0.01) * -18
    crack = pnoise(n, 30, 90)
    crackmask = (np.abs(crack - 0.5) < 0.006) * (pnoise(n, 50) > 0.55)
    v = base + speck + dark - crackmask * 30
    rgb = np.stack([v, v, v * 1.06], -1)
    save("asphalt", rgb, grit * 0.7 + fine * 0.2 - crackmask * 0.8, 2.4, 256)


def make_gravel():
    n = 512
    d, ids = voronoi(n, 900)
    r = 11.0
    pebble = np.clip(1.0 - (d / r) ** 2, 0, 1)
    tone = RNG.uniform(0.6, 1.25, 900)[ids]
    warm = RNG.uniform(0.0, 1.0, 900)[ids]
    col = colorize(warm, [(0, (122, 110, 98)), (0.5, (150, 124, 90)), (1.0, (112, 96, 76))]) * tone[..., None]
    soil = np.array([88, 68, 46]) * (0.8 + 0.4 * fbm(n, 18)[..., None])
    m = np.clip(pebble * 2.2, 0, 1)[..., None]
    rgb = soil * (1 - m) + col * m * (0.7 + 0.3 * pebble[..., None])
    save("gravel", rgb, pebble + fbm(n, 8) * 0.1, 4.0, 256)


def make_crop():
    n = 512
    d, ids = voronoi(n, 2200)
    leaf = np.clip(1.0 - d / 9.0, 0, 1)
    tone = RNG.uniform(0.65, 1.2, 2200)[ids]
    greens = colorize(RNG.random(2200)[ids], [(0, (96, 128, 40)), (1, (140, 164, 56))]) * tone[..., None]
    soil = np.array([96, 74, 48]) * (0.8 + 0.4 * fbm(n, 12)[..., None])
    m = np.clip(leaf * 2.0, 0, 1)[..., None]
    rgb = soil * (1 - m) + greens * m
    rgb = rgb / rgb.mean() * 150.0  # near mid-grey so vertex colours (row tints) dominate
    save("crop", rgb, leaf * 0.8 + fbm(n, 6) * 0.2, 3.5, 256)


def make_bark():
    n = 256
    ridge = pnoise(n, 3.0, 60.0)
    groove = np.sin((ridge * 14.0 + np.arange(n)[None, :] / 9.0) * np.pi)
    groove = norm01(groove)
    blotch = fbm(n, 30)
    rgb = colorize(groove * 0.8 + blotch * 0.2, [(0, (38, 26, 18)), (0.5, (82, 58, 38)), (1, (122, 92, 62))])
    save("bark", rgb, groove, 3.2, 256)


def make_leaves():
    n = 512
    d, ids = voronoi(n, 650)
    clump = np.clip(1.0 - d / 24.0, 0, 1)
    tone = RNG.uniform(0.6, 1.25, 650)[ids]
    greens = colorize(RNG.random(650)[ids], [(0, (40, 84, 34)), (0.5, (70, 120, 44)), (1, (108, 150, 56))]) * tone[..., None]
    gap = np.clip(0.55 + clump * 0.9, 0, 1)[..., None]  # darker towards clump edges
    rgb = greens * gap * (0.85 + 0.3 * fbm(n, 6)[..., None])
    save("leaves", rgb, clump + fbm(n, 5) * 0.2, 3.0, 256)


def make_truck_paint():
    n = 512
    base = np.array([150, 190, 112])
    wear = fbm(n, 70)
    peel = pnoise(n, 14) * 0.6 + pnoise(n, 4) * 0.4
    rgb = base * (0.86 + 0.24 * wear[..., None])
    rust = ((peel > 0.70) & (wear > 0.45))[..., None]
    rgb = np.where(rust, np.array([128, 86, 54]) * (0.8 + 0.4 * pnoise(n, 3)[..., None]), rgb)
    # fine scratches
    yy, xx = np.mgrid[0:n, 0:n]
    scr = np.zeros((n, n))
    for _ in range(26):
        x0, y0 = RNG.integers(0, n, 2)
        ang = RNG.uniform(0, np.pi)
        for t in range(RNG.integers(20, 90)):
            x, y = int(x0 + t * np.cos(ang)) % n, int(y0 + t * np.sin(ang)) % n
            scr[y, x] = 1
    rgb = rgb + scr[..., None] * 45
    grime = np.clip((fbm(n, 24) - 0.55) * 2.2, 0, 1)[..., None]
    rgb = rgb * (1 - 0.35 * grime)
    save("truck_paint", rgb, wear * 0.15 + peel * 0.1 - rust[..., 0] * 0.4, 1.6, 512)


def make_truck_yellow():
    rgb, h = planks(512, 8, [(214, 170, 52)], paint=(226, 186, 58))
    save("truck_yellow", rgb, h, 2.6, 512)


def make_wicker():
    n = 256
    yy, xx = np.mgrid[0:n, 0:n]
    cell = 16
    over = ((xx // cell + yy // cell) % 2).astype(float)
    sx = np.sin(np.pi * (xx % cell) / cell)
    sy = np.sin(np.pi * (yy % cell) / cell)
    h = over * sx + (1 - over) * sy
    tone = RNG.uniform(0.8, 1.2, (n // cell + 1, n // cell + 1))[yy // cell, xx // cell]
    rgb = np.array([112, 66, 40]) * (0.6 + 0.5 * h[..., None]) * tone[..., None] * (0.9 + 0.2 * fbm(n, 12)[..., None])
    save("wicker", rgb, h, 3.0, 256)


def make_strata():
    """Banded sedimentary layers; harder layers stand proud. Horizontal bands, tileable."""
    n = 512
    # 1-D layer profile along y, periodic
    t = np.linspace(0, 1, n, endpoint=False)
    prof = np.zeros(n)
    for k, a in enumerate([1, 2, 3, 5, 8]):
        prof += np.sin(2 * np.pi * (a * t + RNG.random())) / (1.0 + 0.6 * k)
    layer = np.floor(norm01(prof + 0.35 * np.sin(2 * np.pi * 3 * t)) * 6.0) / 6.0
    wob = pnoise(n, 120, 40)  # layers wobble sideways a little
    yy, xx = np.mgrid[0:n, 0:n]
    ywarp = ((yy + (wob - 0.5) * 18) % n).astype(int)
    band = layer[ywarp]
    rgb = colorize(band, [(0.0, (128, 112, 96)), (0.2, (206, 168, 112)), (0.4, (168, 92, 56)), (0.6, (226, 200, 150)), (0.8, (136, 74, 48)), (1.0, (96, 90, 86))])
    rgb *= (0.84 + 0.3 * fbm(n, 5)[..., None])
    frac = ((np.abs(pnoise(n, 3, 70) - 0.5) < 0.006) & (pnoise(n, 60) > 0.55)) * 1.0
    rgb *= (1 - 0.45 * frac)[..., None]
    seam = (np.abs(np.diff(layer, append=layer[0])) > 1e-6)[ywarp]
    rgb *= (1 - 0.40 * seam)[..., None]
    h = band * 0.8 + fbm(n, 4) * 0.25 - frac * 0.5 - seam * 0.3
    save("strata", rgb, h, 3.4, 512)


def make_fabric_normal():
    n = 128
    yy, xx = np.mgrid[0:n, 0:n]
    h = 0.5 + 0.5 * np.sin(2 * np.pi * xx / 4.0) * np.sin(2 * np.pi * yy / 4.0) + 0.2 * RNG.random((n, n))
    Image.fromarray(normal_from_height(h, 1.6)).save(os.path.join(OUT, "fabric_normal.png"), optimize=True)
    print("wrote fabric_normal")


if __name__ == "__main__":
    for fn in (make_wood, make_shingles, make_canvas, make_asphalt, make_gravel, make_crop, make_bark,
               make_leaves, make_truck_paint, make_truck_yellow, make_wicker, make_strata, make_fabric_normal):
        fn()
