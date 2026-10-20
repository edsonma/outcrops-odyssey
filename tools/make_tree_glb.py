#!/usr/bin/env python3
"""Generate assets/models/tree.glb (trunk + clustered canopy) with plain numpy.

A stand-in for a Blender export: shows the exact glTF 2.0 binary layout Bevy loads
(`models/tree.glb#Scene0`). Replace the file with your own .glb and nothing else changes.
usage: python3 -I tools/make_tree_glb.py <output.glb> [texture dir with bark_*/leaves_* maps]
"""
import json, os, struct, sys
import numpy as np

rng = np.random.default_rng(7)

def icosphere(sub):
    t = (1 + 5 ** 0.5) / 2
    v = [(-1, t, 0), (1, t, 0), (-1, -t, 0), (1, -t, 0), (0, -1, t), (0, 1, t), (0, -1, -t), (0, 1, -t),
         (t, 0, -1), (t, 0, 1), (-t, 0, -1), (-t, 0, 1)]
    f = [(0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11), (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6),
         (7, 1, 8), (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9), (4, 9, 5), (2, 4, 11), (6, 2, 10),
         (8, 6, 7), (9, 8, 1)]
    v = [np.array(p, float) / np.linalg.norm(p) for p in v]
    for _ in range(sub):
        cache, nf = {}, []
        def mid(a, b):
            k = tuple(sorted((a, b)))
            if k not in cache:
                m = (v[a] + v[b]) / 2
                v.append(m / np.linalg.norm(m)); cache[k] = len(v) - 1
            return cache[k]
        for a, b, c in f:
            ab, bc, ca = mid(a, b), mid(b, c), mid(c, a)
            nf += [(a, ab, ca), (b, bc, ab), (c, ca, bc), (ab, bc, ca)]
        f = nf
    return np.array(v), np.array(f)

def flat(verts, faces, uvs=None):
    """Unindexed triangles with flat normals (outward-wound) and UVs (planar projection if none given)."""
    tri = verts[faces]                       # (n,3,3)
    n = np.cross(tri[:, 1] - tri[:, 0], tri[:, 2] - tri[:, 0])
    n /= np.linalg.norm(n, axis=1, keepdims=True) + 1e-9
    pos = tri.reshape(-1, 3).astype(np.float32)
    if uvs is None:  # seam-free projection, fine for noise-like leaf textures
        uv = np.stack([pos[:, 0] * 0.45 + pos[:, 2] * 0.2, pos[:, 1] * 0.45 + pos[:, 2] * 0.35], 1)
    else:
        uv = uvs[faces].reshape(-1, 2)
    return pos, np.repeat(n, 3, axis=0).astype(np.float32), uv.astype(np.float32)

def canopy():
    v, f = icosphere(2)
    out_p, out_n, out_u = [], [], []
    centers = [(0, 3.6, 0, 1.5), (0.9, 3.2, 0.4, 1.1), (-0.8, 3.3, -0.5, 1.15), (0.2, 4.4, -0.3, 1.0), (-0.3, 3.0, 0.9, 0.95)]
    for cx, cy, cz, r in centers:
        noise = 1 + 0.18 * rng.standard_normal(len(v))
        p = v * noise[:, None] * r + np.array([cx, cy, cz])
        p[:, 1] *= 0.85
        a, b, c = flat(p, f)
        out_p.append(a); out_n.append(b); out_u.append(c)
    return np.concatenate(out_p), np.concatenate(out_n), np.concatenate(out_u)

def trunk(seg=9):
    pts, uvs, faces = [], [], []
    rings = [(0.0, 0.34), (1.2, 0.26), (2.4, 0.18), (3.4, 0.12)]
    for y, r in rings:
        wob = [1 + 0.06 * rng.standard_normal() for _ in range(seg)]
        wob.append(wob[0])  # the seam vertex repeats the first one so the texture wraps cleanly
        for i in range(seg + 1):
            a = 2 * np.pi * (i % seg) / seg
            pts.append((r * np.cos(a) * wob[i], y, r * np.sin(a) * wob[i]))
            uvs.append((i / seg * 1.5, y / 1.7))
    w = seg + 1
    for k in range(len(rings) - 1):
        for i in range(seg):
            a, b = k * w + i, k * w + i + 1
            c, d = a + w, b + w
            faces += [(a, d, c), (a, b, d)]
    return flat(np.array(pts), np.array(faces), np.array(uvs))

def pack(parts, tex_dir=None):
    blob, views, accs, meshes, nodes = bytearray(), [], [], [], []
    images, textures = [], []

    def add_image(path, mime):
        off = len(blob)
        data = open(path, "rb").read()
        blob.extend(data)
        blob.extend(b"\0" * (-len(blob) % 4))
        views.append({"buffer": 0, "byteOffset": off, "byteLength": len(data)})
        images.append({"bufferView": len(views) - 1, "mimeType": mime})
        textures.append({"source": len(images) - 1})
        return len(textures) - 1

    def pbr(name, color, rough, tex, double=False):
        m = {"name": name, "pbrMetallicRoughness": {"baseColorFactor": color, "metallicFactor": 0, "roughnessFactor": rough}}
        if tex and tex_dir:
            m["pbrMetallicRoughness"]["baseColorTexture"] = {"index": add_image(os.path.join(tex_dir, f"{tex}_albedo.jpg"), "image/jpeg")}
            m["normalTexture"] = {"index": add_image(os.path.join(tex_dir, f"{tex}_normal.png"), "image/png")}
            m["pbrMetallicRoughness"]["baseColorFactor"] = [1, 1, 1, 1] if tex != "leaves" else [0.85, 1.0, 0.8, 1]
        if double:
            m["doubleSided"] = True
        return m

    mats = [pbr("bark", [0.30, 0.20, 0.12, 1], 0.95, "bark"), pbr("leaves", [0.20, 0.38, 0.12, 1], 0.8, "leaves", True)]
    for name, (pos, nor, uv), m in parts:
        ids = []
        for arr, kind in ((pos, "POSITION"), (nor, "NORMAL"), (uv, "TEXCOORD_0")):
            off = len(blob); blob.extend(arr.tobytes())
            views.append({"buffer": 0, "byteOffset": off, "byteLength": arr.nbytes, "target": 34962})
            acc = {"bufferView": len(views) - 1, "componentType": 5126, "count": len(arr), "type": "VEC2" if kind == "TEXCOORD_0" else "VEC3"}
            if kind == "POSITION":
                acc["min"], acc["max"] = arr.min(0).tolist(), arr.max(0).tolist()
            accs.append(acc); ids.append(len(accs) - 1)
        meshes.append({"name": name, "primitives": [{"attributes": {"POSITION": ids[0], "NORMAL": ids[1], "TEXCOORD_0": ids[2]}, "material": m}]})
        nodes.append({"name": name, "mesh": len(meshes) - 1})
    root = {"name": "tree", "children": list(range(len(nodes)))}
    nodes.append(root)
    g = {"asset": {"version": "2.0", "generator": "outcrops tools/make_tree_glb.py"},
         "scene": 0, "scenes": [{"nodes": [len(nodes) - 1]}], "nodes": nodes, "meshes": meshes,
         "materials": mats, "accessors": accs, "bufferViews": views, "buffers": [{"byteLength": len(blob)}]}
    if images:
        g["images"], g["textures"] = images, textures
    return g, bytes(blob)

g, blob = pack([("trunk", trunk(), 0), ("canopy", canopy(), 1)], sys.argv[2] if len(sys.argv) > 2 else None)
js = json.dumps(g, separators=(",", ":")).encode()
js += b" " * (-len(js) % 4)
blob += b"\0" * (-len(blob) % 4)
total = 12 + 8 + len(js) + 8 + len(blob)
with open(sys.argv[1], "wb") as f:
    f.write(struct.pack("<4sII", b"glTF", 2, total))
    f.write(struct.pack("<I4s", len(js), b"JSON")); f.write(js)
    f.write(struct.pack("<I4s", len(blob), b"BIN\0")); f.write(blob)
print("wrote", sys.argv[1], total, "bytes")
