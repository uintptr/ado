#!/usr/bin/env python3
"""Local dev server for the ado webapp.

Runs the full stack on a SINGLE origin (no Docker, no nginx), mirroring how
production is served:

  1. spawns `ttyd -W` wrapping `ado --headless` (the stdin/stdout backend)
  2. serves the static `www/` frontend on http://HOST:PORT
  3. reverse-proxies the WebSocket at /ado/ws -> ttyd's /ws

Same-origin matters: browsers silently refuse a cross-origin WebSocket to a
different port (Firefox won't even open the socket), so the frontend must reach
the ws on its own origin — which is also what nginx provides in production via
the default /ado/ws path. No client config/injection is needed.

Usage:
    cargo build
    test/serve_local.py
    test/serve_local.py --port 9000
    test/serve_local.py --ado-bin ./target/release/ado --config ./config.toml
"""

from __future__ import annotations

import argparse
import http.server
import signal
import socket
import subprocess
import sys
import threading
import time
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
WS_PATH = "/ado/ws"


def wait_for_port(host: str, port: int, timeout: float = 5.0) -> bool:
    """Block until something is accepting connections on host:port."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with socket.create_connection((host, port), timeout=0.25):
                return True
        except OSError:
            time.sleep(0.1)
    return False


def port_in_use(host: str, port: int) -> bool:
    """True if something is already accepting connections on host:port."""
    try:
        with socket.create_connection((host, port), timeout=0.25):
            return True
    except OSError:
        return False


def make_handler(www_dir: Path, ttyd_host: str, ttyd_port: int):
    class Handler(http.server.SimpleHTTPRequestHandler):
        def __init__(self, *args, **kwargs):
            super().__init__(*args, directory=str(www_dir), **kwargs)

        def log_message(self, fmt, *args):
            sys.stderr.write(f"[web] {self.address_string()} {fmt % args}\n")

        def end_headers(self):
            # Never cache in dev — avoids stale JS modules after edits.
            self.send_header("Cache-Control", "no-store, must-revalidate")
            super().end_headers()

        def do_GET(self):
            if self.path == WS_PATH and self.headers.get("Upgrade", "").lower() == "websocket":
                self._proxy_ws()
            else:
                super().do_GET()

        def _proxy_ws(self):
            """Relay the WebSocket between the browser and ttyd's /ws.

            Transparent byte relay: we replay the handshake to ttyd with the
            path rewritten to /ws, then pump raw bytes both ways. The browser
            -> ttyd direction reads via self.rfile.read1 (NOT the raw socket)
            because the browser pipelines its first ws frame right after the
            handshake, so those bytes are already buffered in rfile.
            """
            try:
                upstream = socket.create_connection((ttyd_host, ttyd_port))
            except OSError as e:
                self.send_error(502, f"ttyd not reachable: {e}")
                return

            request = [f"GET /ws {self.request_version}"]
            request += [f"{k}: {v}" for k, v in self.headers.items()]
            request += ["", ""]
            upstream.sendall("\r\n".join(request).encode("latin-1"))

            client = self.connection
            self.close_connection = True

            def shutdown_both():
                for s in (client, upstream):
                    try:
                        s.shutdown(socket.SHUT_RDWR)
                    except OSError:
                        pass

            def to_upstream():
                try:
                    while True:
                        chunk = self.rfile.read1(65536)
                        if not chunk:
                            break
                        upstream.sendall(chunk)
                except OSError:
                    pass
                finally:
                    shutdown_both()

            def to_client():
                try:
                    while True:
                        chunk = upstream.recv(65536)
                        if not chunk:
                            break
                        client.sendall(chunk)
                except OSError:
                    pass
                finally:
                    shutdown_both()

            t = threading.Thread(target=to_client, daemon=True)
            t.start()
            to_upstream()
            t.join()
            upstream.close()

    return Handler


def main() -> int:
    parser = argparse.ArgumentParser(description="Local dev server for the ado webapp.")
    parser.add_argument("--port", type=int, default=8080, help="frontend port (default: 8080)")
    parser.add_argument("--ttyd-port", type=int, default=7681, help="ttyd backend port (default: 7681)")
    parser.add_argument("--host", default="127.0.0.1", help="bind address (default: 127.0.0.1)")
    parser.add_argument("--ado-bin", default=str(REPO_ROOT / "target" / "debug" / "ado"), help="path to the ado binary")
    parser.add_argument("--config", default=str(REPO_ROOT / "config.toml"), help="ado config file")
    parser.add_argument("--www", default=str(REPO_ROOT / "www"), help="static webapp directory")
    args = parser.parse_args()

    ado_bin = Path(args.ado_bin)
    config = Path(args.config)
    www_dir = Path(args.www)

    if not ado_bin.exists():
        print(f"error: ado binary not found at {ado_bin}\n  build it first: cargo build", file=sys.stderr)
        return 1
    if not config.exists():
        print(f"error: config not found at {config}", file=sys.stderr)
        return 1
    if not www_dir.is_dir():
        print(f"error: www dir not found at {www_dir}", file=sys.stderr)
        return 1

    for label, port in (("frontend", args.port), ("ttyd", args.ttyd_port)):
        if port_in_use(args.host, port):
            print(
                f"error: {label} port {port} is already in use.\n"
                f"  a previous run may have leaked a process — check: lsof -nP -iTCP:{port} -sTCP:LISTEN\n"
                f"  or pick another port (--port / --ttyd-port).",
                file=sys.stderr,
            )
            return 1

    ttyd = subprocess.Popen(
        [
            "ttyd",
            "-p", str(args.ttyd_port),
            "-i", args.host,
            "-W",  # writable: let the webapp send stdin
            str(ado_bin), "--headless", "--config-file", str(config),
        ]
    )

    if not wait_for_port(args.host, args.ttyd_port):
        print("error: ttyd did not come up (is it installed?)", file=sys.stderr)
        ttyd.terminate()
        return 1

    handler = make_handler(www_dir, args.host, args.ttyd_port)
    httpd = http.server.ThreadingHTTPServer((args.host, args.port), handler)

    print("ado dev server ready:")
    print(f"  open      : http://{args.host}:{args.port}")
    print(f"  ws        : {WS_PATH} -> ttyd {args.host}:{args.ttyd_port} (same origin)")
    print(f"  ado       : {ado_bin} --headless --config-file {config}")
    print("Ctrl-C to stop.")

    # Translate SIGTERM (e.g. `kill <pid>`) into the same graceful shutdown as
    # Ctrl-C, so the finally block runs and ttyd is never orphaned.
    def on_sigterm(*_):
        raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, on_sigterm)

    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nshutting down...")
    finally:
        httpd.server_close()
        ttyd.terminate()
        try:
            ttyd.wait(timeout=3)
        except subprocess.TimeoutExpired:
            ttyd.kill()

    return 0


if __name__ == "__main__":
    sys.exit(main())
