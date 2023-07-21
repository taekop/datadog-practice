import logging
import requests
from datadog import initialize, statsd
from ddtrace import patch, tracer, config
from http.server import BaseHTTPRequestHandler, HTTPServer

backendUrl = "http://backend:8081"

hostName = "0.0.0.0"
serverPort = 8080

# enable tracer for custom tracing
tracer.configure(hostname="datadog-agent", port=8126)

# enable tracer for requests with service name overriden as frontend
config.requests["service"] = "frontend"
patch(requests=True)

# enable logging
FORMAT = (
    "%(asctime)s %(levelname)s [%(name)s] [%(filename)s:%(lineno)d] "
    "[dd.service=%(dd.service)s dd.env=%(dd.env)s "
    "dd.version=%(dd.version)s "
    "dd.trace_id=%(dd.trace_id)s dd.span_id=%(dd.span_id)s] "
    "- %(message)s"
)
patch(logging=True)
logging.basicConfig(level=logging.INFO, format=FORMAT)


# enable dogstatsd to record custom metrics
initialize(statsd_host="datadog-agent", statsd_port=8125, hostname_from_config=False)


def increment(val):
    with tracer.trace("increment"):
        response = requests.post(
            backendUrl,
            json={
                "jsonrpc": "2.0",
                "id": 1,
                "method": "increment",
                "params": [val],
            },
        )
        return response.json()["result"]


class Frontend(BaseHTTPRequestHandler):
    def do_GET(self):
        with tracer.trace("GET /"):
            logging.info("GET")
            statsd.increment("web.request.count")

            ret = increment(1)

            self.send_response(200)
            self.send_header("Content-type", "text/html")
            self.end_headers()
            self.wfile.write(
                bytes(f"<html><body>Hello world {ret}</body></html>", "utf-8")
            )

    def log_message(self, format, *args):
        # ignore logging
        pass


if __name__ == "__main__":
    server = HTTPServer((hostName, serverPort), Frontend)
    server.serve_forever()