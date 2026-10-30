#!/usr/bin/env python3
"""Measure a release TUI in a 120x40 PTY (Linux/macOS, stdlib only)."""

import argparse
import fcntl
import json
import os
import platform
import pty
import re
import select
import statistics
import struct
import subprocess
import tempfile
import termios
import time
from pathlib import Path


def usage(pid):
    if platform.system() == "Linux":
        stat = Path(f"/proc/{pid}/stat").read_text().rsplit(")", 1)[1].split()
        cpu = (int(stat[11]) + int(stat[12])) / os.sysconf("SC_CLK_TCK")
        rss = int(stat[21]) * os.sysconf("SC_PAGE_SIZE")
        return cpu, rss
    fields = subprocess.check_output(
        ["ps", "-o", "time=,rss=", "-p", str(pid)], text=True
    ).split()
    parts = [float(part) for part in fields[0].split(":")]
    cpu = 0.0
    for part in parts:
        cpu = cpu * 60 + part
    return cpu, int(fields[1]) * 1024


def sample(binary, seconds, refresh_ms):
    config = tempfile.TemporaryDirectory(prefix="toppy-measure-")
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 120, 0, 0))
    start = time.monotonic()
    proc = subprocess.Popen(
        [str(binary), "--refresh-rate", str(refresh_ms)],
        stdin=slave, stdout=slave, stderr=slave,
        env={**os.environ, "TERM": "xterm-256color", "XDG_CONFIG_HOME": config.name},
    )
    try:
        output = bytearray()
        deadline = start + 30
        while b"help" not in re.sub(rb"\x1b\[[0-?]*[ -/]*[@-~]", b"", output):
            if time.monotonic() >= deadline or proc.poll() is not None:
                raise RuntimeError(f"first frame not received: {output!r}")
            if select.select([master], [], [], 0.1)[0]:
                output.extend(os.read(master, 65536))
        startup_ms = (time.monotonic() - start) * 1000
        cpu_start, _ = usage(proc.pid)
        idle_start = time.monotonic()
        rss_samples = []
        while time.monotonic() - idle_start < seconds:
            if select.select([master], [], [], 0.05)[0]:
                os.read(master, 65536)
            _, rss = usage(proc.pid)
            rss_samples.append(rss)
        elapsed = time.monotonic() - idle_start
        cpu_end, _ = usage(proc.pid)
        os.write(master, b"q")
        proc.wait(timeout=5)
        if proc.returncode:
            raise RuntimeError(f"monitor exited with {proc.returncode}")
        return {"startup_ms": startup_ms,
                "idle_cpu_percent_one_core": 100 * (cpu_end - cpu_start) / elapsed,
                "idle_rss_bytes": statistics.median(rss_samples)}
    finally:
        if proc.poll() is None:
            proc.kill()
            proc.wait()
        os.close(master)
        os.close(slave)
        config.cleanup()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--seconds", type=float, default=10)
    parser.add_argument("--samples", type=int, default=3)
    parser.add_argument("--refresh-ms", type=int, default=1500)
    args = parser.parse_args()
    if args.seconds <= 0 or args.samples < 1:
        parser.error("seconds and samples must be positive")
    binary = args.binary.resolve()
    results = [sample(binary, args.seconds, args.refresh_ms) for _ in range(args.samples)]
    print(json.dumps({"platform": platform.platform(), "binary_bytes": binary.stat().st_size,
                      "logical_cpus": os.cpu_count(), "refresh_ms": args.refresh_ms,
                      "seconds_per_sample": args.seconds, "samples": results,
                      "median": {key: statistics.median(row[key] for row in results)
                                 for key in results[0]}}, indent=2))


if __name__ == "__main__":
    main()
