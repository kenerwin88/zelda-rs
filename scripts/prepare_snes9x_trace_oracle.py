#!/usr/bin/env python3
"""Apply boot-contract instrumentation to a clean exact Snes9x 1.63 checkout."""

from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


REVISION = "185488cd83aaf274752a742c94d45561cbecb7af"
DMA_HELPER = '''\
\nextern uint64 zelda3_vram_trace_frame;
static inline void zelda3_trace_vram_dma(uint8 channel, SDMA *d, int32 count)
{
\tconst char *path = getenv("ZELDA3_SNES9X_VRAM_TRACE");
\tif (!path || (d->BAddress != 0x04 && d->BAddress != 0x18 && d->BAddress != 0x19)) return;
\tstatic FILE *trace = NULL;
\tif (!trace) trace = fopen(path, "a");
\tif (!trace) return;
\tfprintf(trace, "%llu dma %u %02x:%04x %02x %d %u %u %u %u\\n", (unsigned long long) zelda3_vram_trace_frame, (unsigned) channel, (unsigned) d->ABank, (unsigned) d->AAddress, (unsigned) d->BAddress, (int) count, (unsigned) d->TransferMode, (unsigned) d->AAddressFixed, (unsigned) d->AAddressDecrement, (unsigned) PPU.VMA.Address);
\tfflush(trace);
\tfprintf(trace, "%llu dma_pc %06x\\n", (unsigned long long) zelda3_vram_trace_frame, (unsigned) (Registers.PBPC & 0xffffff));
\tfflush(trace);
}
'''

LIBRETRO_HELPER = '''\
\nuint64 zelda3_vram_trace_frame = 0;
void zelda3_trace_wram_write(uint8 byte, uint32 address)
{
    const char *path = getenv("ZELDA3_SNES9X_VRAM_TRACE"); if (!path || !*path) return;
    FILE *trace = fopen(path, "a"); if (!trace) return;
    fprintf(trace, "%llu wram %04x %02x %06x\\n", (unsigned long long) zelda3_vram_trace_frame, (unsigned) address, (unsigned) byte, (unsigned) (Registers.PBPC & 0xffffff)); fclose(trace);
}
static void zelda3_trace_frame_state(const char *stage)
{
    const char *path = getenv("ZELDA3_SNES9X_VRAM_TRACE"); if (!path || !*path) return;
    FILE *trace = fopen(path, "a"); if (!trace) return;
    fprintf(trace, "%llu state-%s %02x %02x %02x %02x %02x %02x %02x %02x\\n", (unsigned long long) zelda3_vram_trace_frame, stage, Memory.RAM[0x0010], Memory.RAM[0x0011], Memory.RAM[0x0012], Memory.RAM[0x0013], Memory.RAM[0x0014], Memory.RAM[0x0015], Memory.RAM[0x0016], Memory.RAM[0x0017]); fclose(trace);
}
'''


def replace_once(path: Path, before: str, after: str) -> None:
    text = path.read_text()
    if before not in text:
        raise RuntimeError(f"expected exact Snes9x source anchor missing: {path}: {before[:48]!r}")
    path.write_text(text.replace(before, after, 1))


def run(*args: str, cwd: Path) -> None:
    subprocess.run(args, cwd=cwd, check=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path, help="clean checkout of Snes9x libretro core")
    parser.add_argument("--build", action="store_true", help="build snes9x_libretro.dylib after applying the trace patch")
    args = parser.parse_args()
    source = args.source.resolve()
    revision = subprocess.check_output(("git", "rev-parse", "HEAD"), cwd=source, text=True).strip()
    if revision != REVISION:
        parser.error(f"expected Snes9x revision {REVISION}, got {revision}")
    if subprocess.check_output(("git", "status", "--porcelain"), cwd=source, text=True):
        parser.error("trace source must be clean")
    replace_once(source / "dma.cpp", '#include "spc7110emu.h"', '#include "spc7110emu.h"\n#include <cstdio>\n#include <cstdlib>')
    replace_once(source / "dma.cpp", 'static inline bool8 HDMAReadLineCount (int);', 'static inline bool8 HDMAReadLineCount (int);' + DMA_HELPER)
    replace_once(source / "dma.cpp", '\tif (count == 0)\n\t\tcount = 0x10000;', '\tif (count == 0)\n\t\tcount = 0x10000;\n\tzelda3_trace_vram_dma(Channel, d, count);')
    replace_once(source / "getset.h", 'inline void S9xSetByte', 'extern void zelda3_trace_wram_write(uint8 Byte, uint32 Address);\n\ninline void S9xSetByte')
    replace_once(source / "getset.h", '\t\t*(SetAddress + (Address & 0xffff)) = Byte;', '\t\t*(SetAddress + (Address & 0xffff)) = Byte;\n\t\tuint8 *written = SetAddress + (Address & 0xffff);\n\t\tif ((written >= Memory.RAM && written < Memory.RAM + 0x0400) || (written >= Memory.RAM + 0x0800 && written < Memory.RAM + 0x0810))\n\t\t\tzelda3_trace_wram_write(Byte, (uint32) (written - Memory.RAM));')
    replace_once(source / "libretro/libretro.cpp", 'void retro_run()\n{', LIBRETRO_HELPER + '\nvoid retro_run()\n{\n\tzelda3_vram_trace_frame++;')
    replace_once(source / "libretro/libretro.cpp", '    report_buttons();\n    S9xMainLoop();', '    report_buttons();\n    zelda3_trace_frame_state("before");\n    S9xMainLoop();\n    zelda3_trace_frame_state("after");')
    if args.build:
        run("make", "-C", "libretro", "LTO=", "-j4", "snes9x_libretro.dylib", cwd=source)
    print(f"prepared trace oracle at {source}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
