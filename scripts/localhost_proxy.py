#!/usr/bin/env python3
"""Forward Windows localhost ports to Podman/WSL published ports.

On Windows, Podman Machine (WSL backend) often binds published ports inside
the VM (reachable via the WSL eth0 IP) but not on 127.0.0.1. Browsers then
show ERR_CONNECTION_REFUSED for http://localhost:3000.

This relay restores browser access without admin / netsh portproxy.

Usage:
  python scripts/localhost_proxy.py              # auto-detect WSL IP, proxy 3000+80
  python scripts/localhost_proxy.py --ports 3000
  python scripts/localhost_proxy.py --target-host 172.17.x.x --ports 3000 80
"""

from __future__ import annotations

import argparse
import re
import select
import socket
import subprocess
import sys
import threading
from typing import Iterable


def detect_wsl_ip() -> str | None:
    """Best-effort WSL / Podman machine IP on Windows."""
    candidates: list[list[str]] = [
        ["wsl", "-d", "podman-machine-default", "-e", "bash", "-c",
         "ip -4 -o addr show eth0 | awk '{print $4}' | cut -d/ -f1"],
        ["wsl", "-e", "bash", "-c",
         "ip -4 -o addr show eth0 | awk '{print $4}' | cut -d/ -f1"],
        ["wsl", "hostname", "-I"],
    ]
    for cmd in candidates:
        try:
            out = subprocess.check_output(cmd, text=True, stderr=subprocess.DEVNULL, timeout=8)
        except (OSError, subprocess.SubprocessError):
            continue
        for token in re.findall(r"\b\d{1,3}(?:\.\d{1,3}){3}\b", out):
            if not token.startswith("127."):
                return token
    return None


def pipe(a: socket.socket, b: socket.socket) -> None:
    try:
        while True:
            r, _, _ = select.select([a, b], [], [], 120)
            if not r:
                break
            for src in r:
                dst = b if src is a else a
                data = src.recv(65536)
                if not data:
                    return
                dst.sendall(data)
    except OSError:
        pass
    finally:
        for s in (a, b):
            try:
                s.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            try:
                s.close()
            except OSError:
                pass


def handle(client: socket.socket, target_host: str, target_port: int) -> None:
    remote = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        remote.settimeout(10)
        remote.connect((target_host, target_port))
        remote.settimeout(None)
        remote.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        client.setsockopt(socket.IPPROTO_TCP, socket.TCP_NODELAY, 1)
        pipe(client, remote)
    except OSError:
        try:
            client.close()
        except OSError:
            pass
        try:
            remote.close()
        except OSError:
            pass


def serve_one(listen_port: int, target_host: str, target_port: int) -> None:
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", listen_port))
    srv.listen(128)
    print(
        f"PROXY_READY 127.0.0.1:{listen_port} -> {target_host}:{target_port}",
        flush=True,
    )
    while True:
        client, _ = srv.accept()
        threading.Thread(
            target=handle, args=(client, target_host, target_port), daemon=True
        ).start()


def serve_ports(target_host: str, ports: Iterable[int]) -> None:
    threads: list[threading.Thread] = []
    for port in ports:
        t = threading.Thread(
            target=serve_one, args=(port, target_host, port), daemon=True
        )
        t.start()
        threads.append(t)
    # Keep process alive
    for t in threads:
        t.join()


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--target-host", default=None, help="WSL/Podman IP (auto if omitted)")
    p.add_argument(
        "--ports",
        nargs="+",
        type=int,
        default=[3000, 80],
        help="Local ports to forward (default: 3000 80)",
    )
    # Back-compat single-port flags
    p.add_argument("--listen", type=int, default=None)
    p.add_argument("--target-port", type=int, default=None)
    args = p.parse_args()

    host = args.target_host or detect_wsl_ip()
    if not host:
        print(
            "ERROR: could not detect WSL IP. Pass --target-host explicitly.\n"
            "  wsl -d podman-machine-default -e bash -c \"ip -4 -o addr show eth0\"",
            file=sys.stderr,
        )
        return 1

    if args.listen is not None:
        ports = [args.listen]
        # single-port mode may remap target port
        if args.target_port is not None and args.target_port != args.listen:
            serve_one(args.listen, host, args.target_port)
            return 0
    else:
        ports = args.ports

    print(f"target host: {host}", flush=True)
    try:
        serve_ports(host, ports)
    except KeyboardInterrupt:
        return 0
    return 0


if __name__ == "__main__":
    sys.exit(main())
