#!/usr/bin/env python3
"""Super-resolve captured frames with Real-ESRGAN (torch/MPS) for HD-art authoring.

Offline authoring only; not on the game path. Output PNGs are gitignored.
"""
import argparse, os, sys, glob
from PIL import Image
import numpy as np

def _device():
    import torch
    if torch.backends.mps.is_available():
        return "mps"
    return "cuda" if torch.cuda.is_available() else "cpu"

def _load_model(scale, model, cache_dir):
    """Return a callable img(np.uint8 HWC RGB) -> np.uint8 HWC RGB upscaled by `scale`.
    Uses Real-ESRGAN RRDBNet weights (anime by default). Downloads once to cache_dir."""
    import torch
    from basicsr.archs.rrdbnet_arch import RRDBNet
    from realesrgan import RealESRGANer
    os.makedirs(cache_dir, exist_ok=True)
    if model == "anime":
        net = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=6, num_grow_ch=32, scale=4)
        url = "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.2.2.4/RealESRGAN_x4plus_anime_6B.pth"
        name = "RealESRGAN_x4plus_anime_6B.pth"
    else:
        net = RRDBNet(num_in_ch=3, num_out_ch=3, num_feat=64, num_block=23, num_grow_ch=32, scale=4)
        url = "https://github.com/xinntao/Real-ESRGAN/releases/download/v0.1.0/RealESRGAN_x4plus.pth"
        name = "RealESRGAN_x4plus.pth"
    path = os.path.join(cache_dir, name)
    if not os.path.exists(path):
        torch.hub.download_url_to_file(url, path)
    up = RealESRGANer(scale=4, model_path=path, model=net, half=False, device=_device())
    def run(arr):
        out, _ = up.enhance(arr, outscale=scale)
        return out
    return run

def _nearest(arr, scale):
    return np.array(Image.fromarray(arr).resize(
        (arr.shape[1] * scale, arr.shape[0] * scale), Image.NEAREST))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="indir", default="hd_art/capture")
    ap.add_argument("--out", dest="outdir", default="hd_art/sr")
    ap.add_argument("--scale", type=int, default=4)
    ap.add_argument("--model", choices=["anime", "photo"], default="anime")
    ap.add_argument("--cache", default="hd_art/models")
    ap.add_argument("--self-test", action="store_true")
    args = ap.parse_args()

    if args.self_test:
        img = (np.random.default_rng(0).integers(0, 256, (8, 8, 3))).astype(np.uint8)
        out = _nearest(img, args.scale)
        assert out.shape == (8 * args.scale, 8 * args.scale, 3), out.shape
        print("self-test OK:", out.shape)
        return

    os.makedirs(args.outdir, exist_ok=True)
    run = _load_model(args.scale, args.model, args.cache)
    frames = [f for f in sorted(glob.glob(os.path.join(args.indir, "frame_*.png")))
              if "reference_palette" not in os.path.basename(f)]
    if not frames:
        print(f"no frame_*.png in {args.indir}", file=sys.stderr); sys.exit(1)
    for f in frames:
        base = os.path.splitext(os.path.basename(f))[0]
        img = np.array(Image.open(f).convert("RGB"))
        out = run(img)
        dst = os.path.join(args.outdir, f"{base}.x{args.scale}.png")
        Image.fromarray(out).save(dst)
        print("wrote", dst, out.shape)

if __name__ == "__main__":
    main()
