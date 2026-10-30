#!/usr/bin/env python3
"""PTY regression tests; run with a built toppy binary (Linux/macOS)."""

import argparse
import fcntl
import os
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time
import unittest
from pathlib import Path

BINARY = None


class TerminalTests(unittest.TestCase):
    def setUp(self):
        self.config = tempfile.TemporaryDirectory(prefix="toppy-test-")
        self.addCleanup(self.config.cleanup)
        self.master, self.slave = pty.openpty()
        self.addCleanup(os.close, self.slave)
        self.addCleanup(os.close, self.master)
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 120, 0, 0))
        self.original = termios.tcgetattr(self.slave)
        self.output = bytearray()

    def launch(self, stdout=None):
        proc = subprocess.Popen(
            [str(BINARY), "--refresh-rate", "200"], stdin=self.slave,
            stdout=self.slave if stdout is None else stdout, stderr=self.slave,
            env={**os.environ, "TERM": "xterm-256color", "XDG_CONFIG_HOME": self.config.name},
        )
        self.addCleanup(self.stop, proc)
        return proc

    @staticmethod
    def stop(proc):
        if proc.poll() is None:
            proc.kill()
        proc.wait()

    def read(self, seconds=0.05):
        if select.select([self.master], [], [], seconds)[0]:
            self.output.extend(os.read(self.master, 65536))

    def wait_for(self, text, timeout=5):
        deadline = time.monotonic() + timeout
        while text not in self.output and time.monotonic() < deadline:
            self.read()
        self.assertIn(text, self.output)

    def test_refresh_continues_during_continuous_input(self):
        self.launch()
        self.wait_for(b"help")
        marker = f"toppy-refresh-{os.getpid()}"
        os.write(self.master, b"/" + marker.encode() + b"\r")
        # Let the filter draw while the matching process does not exist yet.
        self.wait_for(b"no matching processes")
        worker = subprocess.Popen(["bash", "-c", 'exec -a "$0" sleep 30', marker])
        self.addCleanup(self.stop, worker)
        self.output.clear()
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            os.write(self.master, b"\x1b[B")
            self.read(0.01)
        self.assertTrue(marker.encode() in self.output,
                      "new process never appeared while input kept poll() busy")

    def test_normal_exit_restores_terminal(self):
        proc = self.launch()
        self.wait_for(b"help")
        os.write(self.master, b"q")
        self.assertEqual(proc.wait(timeout=5), 0)
        self.assertEqual(termios.tcgetattr(self.slave), self.original)
        self.read()
        self.assertIn(b"\x1b[?1049l", self.output)

    def test_setup_failure_restores_raw_mode(self):
        # /dev/full accepts the terminal setup call but fails writes after raw mode
        # was enabled. Keep stdin attached to the PTY so raw-mode changes are real.
        if not Path("/dev/full").exists():
            self.skipTest("requires /dev/full")
        with open("/dev/full", "wb", buffering=0) as full:
            proc = self.launch(stdout=full)
            self.assertNotEqual(proc.wait(timeout=5), 0)
        self.assertEqual(termios.tcgetattr(self.slave), self.original)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    args, remaining = parser.parse_known_args()
    BINARY = args.binary.resolve()
    unittest.main(argv=[__file__, *remaining])
