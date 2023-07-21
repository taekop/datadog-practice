import json
from ddtrace import tracer
from ddtrace.propagation.http import HTTPPropagator
from http.server import BaseHTTPRequestHandler, HTTPServer

hostName = "0.0.0.0"
serverPort = 8082

# enable tracer for custom tracing
tracer.configure(hostname="datadog-agent", port=8126)

x = 0


class DB(BaseHTTPRequestHandler):
    def do_POST(self):
        context = HTTPPropagator.extract(self.headers)
        tracer.context_provider.activate(context)
        content_len = int(self.headers.get("Content-Length"))
        post_body = json.loads(self.rfile.read(content_len))
        method = post_body["method"]
        with tracer.trace(method):
            if method == "get":
                self._get(post_body["id"], *post_body["params"])
            elif method == "set":
                self._set(post_body["id"], *post_body["params"])

    def log_message(self, format, *args):
        # ignore logging
        pass

    def _get(self, id):
        global x
        response = {
            "jsonrpc": "2.0",
            "id": id,
            "result": x,
        }
        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        self.wfile.write(bytes(json.dumps(response), "utf-8"))

    def _set(self, id, y):
        global x
        x = y
        response = {
            "jsonrpc": "2.0",
            "id": id,
            "result": x,
        }
        self.send_response(200)
        self.send_header("Content-type", "application/json")
        self.end_headers()
        self.wfile.write(bytes(json.dumps(response), "utf-8"))


if __name__ == "__main__":
    server = HTTPServer((hostName, serverPort), DB)
    server.serve_forever()
