"""Bun project abstraction tests."""

from __future__ import annotations

from harmont._toolchain import bun_install_cmd


def test_bun_install_cmd_latest():
    cmd = bun_install_cmd()
    assert "https://bun.sh/install" in cmd
    assert "BUN_INSTALL=/usr/local" in cmd
    assert "bun-v" not in cmd


def test_bun_install_cmd_version():
    cmd = bun_install_cmd("1.2.0")
    assert "bun-v1.2.0" in cmd
