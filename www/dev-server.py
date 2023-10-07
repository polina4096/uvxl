#!/usr/bin/env python3

import http.server
from http.server import HTTPServer, BaseHTTPRequestHandler
import socketserver

PORT = 8080

Handler = http.server.SimpleHTTPRequestHandler

Handler.extensions_map = {
    '.manifest' : 'text/cache-manifest',
	'.html'     : 'text/html',
    '.png'      : 'image/png',
	'.jpg'      : 'image/jpg',
	'.svg'      : 'image/svg+xml',
	'.css'      : 'text/css',
	'.js'       : 'application/x-javascript',
	''          : 'application/octet-stream', # Default
    }

httpd = socketserver.TCPServer(("", PORT), Handler)

print(f"serving at http://localhost:{PORT}")
httpd.serve_forever()
