# Pprof Performance Analysis Tool Usage Guide

## Overview

Pprof is a built-in performance analysis tool in RobustMQ that generates application performance flame graphs to help developers identify performance bottlenecks and optimization opportunities. It has no dedicated HTTP port — collection is toggled by `[runtime]`'s `pprof_enable`, and the flame graph is exposed via the Admin HTTP API (sharing `http_port`).

## Configuration

Add the following configuration to the `config/server.toml` file:

```toml
[runtime]
pprof_enable = true   # Enable pprof collection, default is false
```

## Usage

### 1. Start Service

Ensure `runtime.pprof_enable = true` in the configuration file, then start the RobustMQ service:

```bash
./bin/robust-server start
```

### 2. Generate Flame Graph

Access in your browser (replace `{http_port}` with the `http_port` configured in `server.toml`, default `58080`):

```
http://127.0.0.1:{http_port}/debug/pprof/flamegraph
```

The system will return a performance flame graph in SVG format. If `pprof_enable` is not set, this endpoint returns a plain-text notice instead of a flame graph.

### 3. Analyze Flame Graph

- **Width**: Time proportion of function calls
- **Height**: Call stack depth
- **Color**: Identification markers for different functions
- **Hot spots**: Areas with larger width indicate performance bottlenecks
