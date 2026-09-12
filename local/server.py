#!/usr/bin/env python3
"""Static file server for the locally built site (stdlib only).

Serves the directory given as the first argument, adding the
application/wasm MIME type and disabling caching so rebuilds
show up on refresh.
"""
import sys
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer


class Handler(SimpleHTTPRequestHandler):
    extensions_map = {
        **SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
    }

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()


if __name__ == "__main__":
    directory = sys.argv[1] if len(sys.argv) > 1 else "."
    port = int(sys.argv[2]) if len(sys.argv) > 2 else 8000
    with ThreadingHTTPServer(("127.0.0.1", port), partial(Handler, directory=directory)) as httpd:
        print(f"Serving {directory} at http://127.0.0.1:{port}")
        httpd.serve_forever()
