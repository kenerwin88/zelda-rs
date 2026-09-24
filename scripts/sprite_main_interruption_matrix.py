#!/usr/bin/env python3
"""Summarize Sprite_Main host ownership from cached Snes9x timing receipts.

This is an offline diagnostic. Native runtime timing must never read receipts.
"""

import argparse
import collections
import json
import subprocess
import sys


def event_value(events, name):
    return next((event[name] for event in events if isinstance(event, dict) and name in event), None)


def has_event(events, name):
    return any(event == name or isinstance(event, dict) and name in event for event in events)


def acceptances(events):
    return [event["NmiAccepted"] for event in events
            if isinstance(event, dict) and "NmiAccepted" in event]


def checkpoint_name(progress):
    if isinstance(progress, str):
        return progress
    return next(iter(progress))


def interruption_rows(receipts):
    """Pair each progress host with the following host's ownership events."""
    pending = None
    for receipt in receipts:
        host = receipt["host_call"]
        events = receipt["semantic"]
        if pending is not None:
            pending["next_host"] = host
            pending["next_acceptances"] = acceptances(events)
            pending["next_sprite_returned"] = has_event(events, "SpriteMainReturned")
            pending["next_progress"] = event_value(events, "SpriteMainProgressed")
            yield pending
            pending = None
        progress = event_value(events, "SpriteMainProgressed")
        if progress is not None:
            pending = {
                "host": host,
                "checkpoint": checkpoint_name(progress),
                "progress": progress,
                "acceptances": acceptances(events),
                "interrupted": event_value(events, "MainLoopInterrupted"),
                "sprite_returned": has_event(events, "SpriteMainReturned"),
            }


def read_receipts(path, through_host):
    command = ["zstd", "-dc", str(path)]
    process = subprocess.Popen(command, stdout=subprocess.PIPE,
                               stderr=subprocess.DEVNULL, text=True)
    previous_had_progress = False
    stopped_at_limit = False
    try:
        for line in process.stdout:
            # host_call is the first JSON field and monotonically increases.
            # Each receipt carries large display/audio arrays; decode only a
            # progress host and its immediate successor.
            host = int(line.split(",", 1)[0].split(":", 1)[1])
            if host > through_host:
                stopped_at_limit = True
                break
            has_progress = '"SpriteMainProgressed"' in line
            if has_progress or previous_had_progress:
                yield json.loads(line)
            previous_had_progress = has_progress
    finally:
        process.stdout.close()
        if stopped_at_limit:
            process.terminate()
        return_code = process.wait()
        if not stopped_at_limit and return_code != 0:
            raise RuntimeError(f"zstd failed with status {return_code}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("receipts", help="original-timing-host-receipts.jsonl.zst")
    parser.add_argument("--through-host", type=int, required=True)
    parser.add_argument("--checkpoint", help="show only one checkpoint class")
    parser.add_argument("--json", action="store_true", help="emit all matching rows")
    args = parser.parse_args()
    rows = [row for row in interruption_rows(read_receipts(args.receipts, args.through_host))
            if args.checkpoint is None or row["checkpoint"] == args.checkpoint]
    if args.json:
        json.dump(rows, sys.stdout, indent=2)
        print()
        return
    counts = collections.Counter(
        (row["checkpoint"], tuple(row["acceptances"]),
         tuple(row["next_acceptances"]), row["next_sprite_returned"])
        for row in rows
    )
    for key, count in sorted(counts.items()):
        examples = [row["host"] for row in rows if
                    (row["checkpoint"], tuple(row["acceptances"]),
                     tuple(row["next_acceptances"]), row["next_sprite_returned"]) == key]
        print(f"{count:4} {key} hosts={examples[:12]}")


if __name__ == "__main__":
    main()
