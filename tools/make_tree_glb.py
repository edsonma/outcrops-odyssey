#!/usr/bin/env python3
"""Generate assets/models/tree.glb (trunk + clustered canopy) with plain numpy.

A stand-in for a Blender export: shows the exact glTF 2.0 binary layout Bevy loads
(`models/tree.glb#Scene0`). Replace the file with your own .glb and nothing else changes.
usage: python3 -I tools/make_tree_glb.py <output.glb>
"""
import json, struct, sys
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

def flat(verts, faces):
    """Unindexed triangles with flat normals (outward-wound)."""
    tri = verts[faces]                       # (n,3,3)
    n = np.cross(tri[:, 1] - tri[:, 0], tri[:, 2] - tri[:, 0])
    n /= np.linalg.norm(n, axis=1, keepdims=True) + 1e-9
    return tri.reshape(-1, 3).astype(np.float32), np.repeat(n, 3, axis=0).astype(np.float32)

def canopy():
    v, f = icosphere(2)
    out_p, out_n = [], []
    centers = [(0, 3.6, 0, 1.5), (0.9, 3.2, 0.4, 1.1), (-0.8, 3.3, -0.5, 1.15), (0.2, 4.4, -0.3, 1.0), (-0.3, 3.0, 0.9, 0.95)]
    for cx, cy, cz, r in centers:
        noise = 1 + 0.18 * rng.standard_normal(len(v))
        p = v * noise[:, None] * r + np.array([cx, cy, cz])
        p[:, 1] *= 0.85
        a, b = flat(p, f)
        out_p.append(a); out_n.append(b)
    return np.concatenate(out_p), np.concatenate(out_n)

def trunk(seg=9):
    pts, faces = [], []
    rings = [(0.0, 0.34), (1.2, 0.26), (2.4, 0.18), (3.4, 0.12)]
    for y, r in rings:
        for i in range(seg):
            a = 2 * np.pi * i / seg
            pts.append((r * np.cos(a) * (1 + 0.06 * rng.standard_normal()), y, r * np.sin(a)))
    for k in range(len(rings) - 1):
        for i in range(seg):
            a, b = k * seg + i, k * seg + (i + 1) % seg
            c, d = a + seg, b + seg
            faces += [(a, d, c), (a, b, d)]
    return flat(np.array(pts), np.array(faces))

def pack(parts):
    blob, views, accs, meshes, nodes = bytearray(), [], [], [], []
    mats = [
        {"name": "bark", "pbrMetallicRoughness": {"baseColorFactor": [0.30, 0.20, 0.12, 1], "metallicFactor": 0, "roughnessFactor": 0.95}},
        {"name": "leaves", "pbrMetallicRoughness": {"baseColorFactor": [0.20, 0.38, 0.12, 1], "metallicFactor": 0, "roughnessFactor": 0.8}, "doubleSided": True},
    ]
    for name, (pos, nor), m in parts:
        ids = []
        for arr, comp_type in ((pos, "POSITION"), (nor, "NORMAL")):
            off = len(blob); blob += arr.tobytes()
            views.append({"buffer": 0, "byteOffset": off, "byteLength": arr.nbytes, "target": 34962})
            acc = {"bufferView": len(views) - 1, "componentType": 5126, "count": len(arr), "type": "VEC3"}
            if comp_type == "POSITION":
                acc["min"], acc["max"] = arr.min(0).tolist(), arr.max(0).tolist()
            accs.append(acc); ids.append(len(accs) - 1)
        meshes.append({"name": name, "primitives": [{"attributes": {"POSITION": ids[0], "NORMAL": ids[1]}, "material": m}]})
        nodes.append({"name": name, "mesh": len(meshes) - 1})
    root = {"name": "tree", "children": list(range(len(nodes)))}
    nodes.append(root)
    g = {"asset": {"version": "2.0", "generator": "outcrops tools/make_tree_glb.py"},
         "scene": 0, "scenes": [{"nodes": [len(nodes) - 1]}], "nodes": nodes, "meshes": meshes,
         "materials": mats, "accessors": accs, "bufferViews": views, "buffers": [{"byteLength": len(blob)}]}
    return g, bytes(blob)

g, blob = pack([("trunk", trunk(), 0), ("canopy", canopy(), 1)])
js = json.dumps(g, separators=(",", ":")).encode()
js += b" " * (-len(js) % 4)
blob += b"\0" * (-len(blob) % 4)
total = 12 + 8 + len(js) + 8 + len(blob)
with open(sys.argv[1], "wb") as f:
    f.write(struct.pack("<4sII", b"glTF", 2, total))
    f.write(struct.pack("<I4s", len(js), b"JSON")); f.write(js)
    f.write(struct.pack("<I4s", len(blob), b"BIN\0")); f.write(blob)
print("wrote", sys.argv[1], total, "bytes")
