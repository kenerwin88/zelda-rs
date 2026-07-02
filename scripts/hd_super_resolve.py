#!/usr/bin/env python3
"""Super-resolve (+ optionally neural-style) captured frames for HD-art authoring.

Offline authoring only; not on the game path. Output PNGs are gitignored.

Pipeline per frame: Real-ESRGAN x`scale`  ->  (optional) fast neural style transfer.
The styled/upscaled frame is written as `frame_<n>.x<scale>.png` so `--slice-hd-cells`
consumes it unchanged. Neural style makes overridden cells OBVIOUSLY a different art
style in-game (the kernel treats the art as detail vs the reference palette and re-lights
it through live CGRAM, so the restyle shows through).
"""
import argparse, os, sys, glob
from PIL import Image
import numpy as np

# Styles supported by the bundled fast-neural-style TransformerNet weights.
STYLES = ("udnie", "mosaic", "candy", "rain_princess")


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


# --- Fast neural style transfer (Johnson et al.); arch matches the classic
# pytorch/examples fast_neural_style pretrained models so their weights load 1:1. ---
def _build_transformer_net():
    import torch

    class ConvLayer(torch.nn.Module):
        def __init__(self, in_c, out_c, kernel_size, stride):
            super().__init__()
            self.pad = torch.nn.ReflectionPad2d(kernel_size // 2)
            self.conv = torch.nn.Conv2d(in_c, out_c, kernel_size, stride)

        def forward(self, x):
            return self.conv(self.pad(x))

    class ResidualBlock(torch.nn.Module):
        def __init__(self, ch):
            super().__init__()
            self.conv1 = ConvLayer(ch, ch, 3, 1)
            self.in1 = torch.nn.InstanceNorm2d(ch, affine=True)
            self.conv2 = ConvLayer(ch, ch, 3, 1)
            self.in2 = torch.nn.InstanceNorm2d(ch, affine=True)
            self.relu = torch.nn.ReLU()

        def forward(self, x):
            y = self.relu(self.in1(self.conv1(x)))
            return self.in2(self.conv2(y)) + x

    class UpsampleConvLayer(torch.nn.Module):
        def __init__(self, in_c, out_c, kernel_size, stride, upsample):
            super().__init__()
            self.upsample = upsample
            self.pad = torch.nn.ReflectionPad2d(kernel_size // 2)
            self.conv = torch.nn.Conv2d(in_c, out_c, kernel_size, stride)

        def forward(self, x):
            x = torch.nn.functional.interpolate(x, mode="nearest", scale_factor=self.upsample)
            return self.conv(self.pad(x))

    class TransformerNet(torch.nn.Module):
        def __init__(self):
            super().__init__()
            self.conv1 = ConvLayer(3, 32, 9, 1)
            self.in1 = torch.nn.InstanceNorm2d(32, affine=True)
            self.conv2 = ConvLayer(32, 64, 3, 2)
            self.in2 = torch.nn.InstanceNorm2d(64, affine=True)
            self.conv3 = ConvLayer(64, 128, 3, 2)
            self.in3 = torch.nn.InstanceNorm2d(128, affine=True)
            self.res1, self.res2, self.res3, self.res4, self.res5 = (ResidualBlock(128) for _ in range(5))
            self.deconv1 = UpsampleConvLayer(128, 64, 3, 1, 2)
            self.in4 = torch.nn.InstanceNorm2d(64, affine=True)
            self.deconv2 = UpsampleConvLayer(64, 32, 3, 1, 2)
            self.in5 = torch.nn.InstanceNorm2d(32, affine=True)
            self.deconv3 = ConvLayer(32, 3, 9, 1)
            self.relu = torch.nn.ReLU()

        def forward(self, x):
            y = self.relu(self.in1(self.conv1(x)))
            y = self.relu(self.in2(self.conv2(y)))
            y = self.relu(self.in3(self.conv3(y)))
            y = self.res5(self.res4(self.res3(self.res2(self.res1(y)))))
            y = self.relu(self.in4(self.deconv1(y)))
            y = self.relu(self.in5(self.deconv2(y)))
            return self.deconv3(y)

    return TransformerNet()


def _load_style(style, cache_dir, weights_path):
    """Return a callable img(np.uint8 HWC RGB) -> styled np.uint8 HWC RGB (same size).
    Weights resolution order: explicit --style-weights, then <cache>/<style>.pth. If
    neither exists, prints where to obtain them and exits (the pretrained fast-neural-style
    models: pytorch/examples fast_neural_style saved_models.zip -> {style}.pth)."""
    import torch
    path = weights_path or os.path.join(cache_dir, f"{style}.pth")
    if not os.path.exists(path):
        print(
            f"style weights not found: {path}\n"
            f"  Provide --style-weights <path-to-{style}.pth>, or place {style}.pth in {cache_dir}.\n"
            f"  Get the pretrained fast-neural-style models (candy/mosaic/rain_princess/udnie):\n"
            f"    git clone --depth 1 https://github.com/pytorch/examples\n"
            f"    python examples/fast_neural_style/download_saved_models.py\n"
            f"    cp examples/fast_neural_style/saved_models/{style}.pth {cache_dir}/",
            file=sys.stderr,
        )
        sys.exit(1)
    dev = _device()
    net = _build_transformer_net()
    state = torch.load(path, map_location=dev)
    # Pretrained checkpoints carry deprecated running_mean/var InstanceNorm buffers; drop them.
    for k in [k for k in list(state.keys()) if k.endswith(("running_mean", "running_var"))]:
        del state[k]
    net.load_state_dict(state)
    net.to(dev).eval()

    def run(arr):
        with torch.no_grad():
            t = torch.from_numpy(arr).permute(2, 0, 1).float().unsqueeze(0).to(dev)
            out = net(t).clamp(0, 255).squeeze(0).permute(1, 2, 0).cpu().numpy()
        return out.astype(np.uint8)

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
    ap.add_argument("--style", choices=("none",) + STYLES, default="none",
                    help="apply fast neural style transfer after SR (makes overrides obviously different art)")
    ap.add_argument("--style-weights", default=None,
                    help="explicit path to the <style>.pth weights (else <cache>/<style>.pth)")
    ap.add_argument("--cache", default="hd_art/models")
    ap.add_argument("--self-test", action="store_true")
    ap.add_argument("--self-test-style", action="store_true",
                    help="torch-side plumbing check for the style net (random weights, no download)")
    args = ap.parse_args()

    if args.self_test:
        img = (np.random.default_rng(0).integers(0, 256, (8, 8, 3))).astype(np.uint8)
        out = _nearest(img, args.scale)
        assert out.shape == (8 * args.scale, 8 * args.scale, 3), out.shape
        print("self-test OK:", out.shape)
        return

    if args.self_test_style:
        import torch
        net = _build_transformer_net().to(_device()).eval()
        img = (np.random.default_rng(0).integers(0, 256, (16, 16, 3))).astype(np.uint8)
        with torch.no_grad():
            t = torch.from_numpy(img).permute(2, 0, 1).float().unsqueeze(0).to(_device())
            out = net(t)
        assert tuple(out.shape) == (1, 3, 16, 16), tuple(out.shape)
        print("self-test-style OK:", tuple(out.shape), "device", _device())
        return

    os.makedirs(args.outdir, exist_ok=True)
    run = _load_model(args.scale, args.model, args.cache)
    stylize = _load_style(args.style, args.cache, args.style_weights) if args.style != "none" else None
    frames = [f for f in sorted(glob.glob(os.path.join(args.indir, "frame_*.png")))
              if "reference_palette" not in os.path.basename(f)]
    if not frames:
        print(f"no frame_*.png in {args.indir}", file=sys.stderr); sys.exit(1)
    for f in frames:
        base = os.path.splitext(os.path.basename(f))[0]
        img = np.array(Image.open(f).convert("RGB"))
        out = run(img)
        if stylize is not None:
            out = stylize(out)
        dst = os.path.join(args.outdir, f"{base}.x{args.scale}.png")
        Image.fromarray(out).save(dst)
        print("wrote", dst, out.shape, f"style={args.style}")


if __name__ == "__main__":
    main()
