# Datadog Practice

This repository contains a practice project that demonstrates the integration of the Rust library jsonrpsee with Datadog and Python servers.

## Getting Started

Starting a local server at 127.0.0.1:8080.

```shell
$ docker compose up --build
```

## Components Overview

![Trace](assets/trace.png)

```
 Frontend                                                                  Backend                                                                  DB                 
┌───────────────────────┐          Http Request Headers                   ┌───────────────────────┐         Http Request Headers                   ┌───────────────────────┐
│                       │         ┌─────────────────────────────┐         │                       │        ┌─────────────────────────────┐         │                       │
│   TraceContext        │         │ x-datadog-trace-id          │         │   TraceContext        │        │ x-datadog-trace-id          │         │   TraceContext        │
│ ┌───────────────────┐ │         │                             │         │ ┌───────────────────┐ │        │                             │         │ ┌───────────────────┐ │
│ │ TraceId           │ │         │ x-datadog-parent-id         │         │ │ TraceId           │ │        │ x-datadog-parent-id         │         │ │ TraceId           │ │
│ │                   │ │         │                             │         │ │                   │ │        │                             │         │ │                   │ │
│ │ SpanId            │ │ Inject  │ x-datadog-sampling-priority │ Extract │ │ SpanId            │ │Inject  │ x-datadog-sampling-priority │ Extract │ │ SpanId            │ │
│ │                   ├─┼────────>│                             ├─────────┼>│                   │ ┼───────>│                             ├─────────┼>│                   │ │
│ │ TraceFlags        │ │         │ x-datadog-tags              │         │ │ TraceFlags        │ │        │ x-datadog-tags              │         │ │ TraceFlags        │ │
│ │                   │ │         │                             │         │ │                   │ │        │                             │         │ │                   │ │
│ │ TraceState        │ │         │ traceparent                 │         │ │ TraceState        │ │        │ traceparent                 │         │ │ TraceState        │ │
│ └───────────────────┘ │         │                             │         │ └───────────────────┘ │        │                             │         │ └───────────────────┘ │
│                       │         │ tracestate                  │         │                       │        │ tracestate                  │         │                       │
└───────────────────────┘         └─────────────────────────────┘         └───────────────────────┘        └─────────────────────────────┘         └───────────────────────┘
```

## Log Pipeline

Due to the absence of a default Rust log pipeline in Datadog, it becomes necessary to configure a custom pipeline specifically for Rust. Below, you'll find an example to help you set it up.

| Pipeline          | `source:rust`                                                                                                                                                                             |
| ----------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Grok Parser       | `rust_format %{date("yyyy-MM-dd'T'HH:mm:ss.SSSZZ"):timestamp} %{word:levelname} \[%{data::keyvalue}\] %{data:code.namespace}:%{data:code.filename}:%{data:code.lineno} - %{data:message}` |
| Date Remapper     | Define `timestamp` as the official date of the log                                                                                                                                        |
| Status Remapper   | Define  `levelname`  as the official status of the log                                                                                                                                    |
| Message Remapper  | Define  `message`  as the official message of the log                                                                                                                                     |
| Trace Id Remapper | Define  `dd.trace_id`  as the official trace ID of the log                                                                                                                                |
| Remapper          | Map attribute  `dd.env`  to tag  `env`                                                                                                                                                    |
| Remapper          | Map attribute  `dd.version`  to tag  `version`                                                                                                                                            |
| Service Remapper  | Define  `dd.service`  as the official service of the log                                                                                                                                  |

## Future Improvements
- Trace Naming in Backend
- Metrics Integration
  
## Disclaimer
This project is purely for educational and practice purposes.

Please feel free to explore the code, experiment, and make modifications to further your learning and understanding of the technologies involved.

Happy coding! 🚀
