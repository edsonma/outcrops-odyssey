#!/usr/bin/env python3
"""Convert Poly Haven 2K texture packs (jpg + EXR) into small web-friendly maps for Bevy.

usage (needs: pip install opencv-python-headless numpy OpenEXR):
  python3 -I tools/convert_textures.py <dir containing extracted packs> <output dir>
 - diffuse  -> <name>_albedo.jpg   (sRGB)
 - normal   -> <name>_normal.png   (OpenGL convention, linear)
 - rough    -> <name>_mr.jpg       (glTF metallic-roughness packing: G = roughness, B = metallic = 0)
"""
import os, sys
import cv2
import numpy as np

SRC, DST = sys.argv[1], sys.argv[2]
# name -> (pack dir, file stem, albedo size, normal/rough size)
PACKS = {
    "grass": ("sparse_grass", "sparse_grass", 1024, 512),
    "rock": ("rock_face_03", "rock_face_03", 1024, 512),
    "cliff": ("cliff_side", "cliff_side", 1024, 512),
    "moss": ("aerial_rocks_04", "aerial_rocks_04", 1024, 512),
}

def read_exr(path):
    """Float32 HxWxC array in RGB order. Needs `pip install OpenEXR`."""
    import OpenEXR
    with OpenEXR.File(path) as f:
        ch = f.channels()
        if "RGB" in ch:
            return np.asarray(ch["RGB"].pixels, dtype=np.float32)
        key = "Y" if "Y" in ch else sorted(ch.keys())[0]
        return np.asarray(ch[key].pixels, dtype=np.float32)

def read(path, flags=cv2.IMREAD_UNCHANGED):
    img = cv2.imread(path, flags)
    if img is None:
        raise SystemExit(f"cannot read {path}")
    return img

os.makedirs(DST, exist_ok=True)
for name, (pack, stem, s_alb, s_aux) in PACKS.items():
    tex = os.path.join(SRC, pack, "textures")
    alb = cv2.resize(read(f"{tex}/{stem}_diff_2k.jpg"), (s_alb, s_alb), interpolation=cv2.INTER_AREA)
    cv2.imwrite(f"{DST}/{name}_albedo.jpg", alb, [cv2.IMWRITE_JPEG_QUALITY, 82])

    nor = read_exr(f"{tex}/{stem}_nor_gl_2k.exr")[..., :3]  # RGB float 0..1
    nor = cv2.resize(nor, (s_aux, s_aux), interpolation=cv2.INTER_AREA)
    cv2.imwrite(f"{DST}/{name}_normal.png", cv2.cvtColor((np.clip(nor, 0, 1) * 255).astype(np.uint8), cv2.COLOR_RGB2BGR))

    rp = f"{tex}/{stem}_rough_2k.exr"
    rough = read_exr(rp) if os.path.exists(rp) else read(rp.replace(".exr", ".jpg")).astype(np.float32)
    if rough.ndim == 3:
        rough = rough[..., 0]
    if rough.max() > 1.5:
        rough /= 255.0
    rough = cv2.resize(rough, (s_aux, s_aux), interpolation=cv2.INTER_AREA)
    mr = np.zeros((s_aux, s_aux, 3), np.uint8)   # BGR: B=metallic(0) G=roughness R=occlusion(unused)
    mr[..., 1] = (np.clip(rough, 0, 1) * 255).astype(np.uint8)
    mr[..., 2] = 255
    cv2.imwrite(f"{DST}/{name}_mr.jpg", mr, [cv2.IMWRITE_JPEG_QUALITY, 85])
    print("ok", name)
