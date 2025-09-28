# Pprof Performance Analysis Tool Usage Guide

## Overview

Pprof is a built-in performance analysis tool in RobustMQ that generates application performance flame graphs to help developers identify performance bottlenecks and optimization opportunities.

## Configuration

Add the following configuration to the `config/server.toml` file:

```toml
[p_prof]
enable = true      # Enable pprof functionality
port = 6777        # HTTP service port
frequency = 1000   # Sampling frequency (Hz)
```

### Configuration Parameters

- `enable`: Whether to enable pprof monitoring, default is `false`
- `port`: HTTP server listening port, default is `6060`
- `frequency`: Performance sampling frequency in Hz, default is `100`

## Usage

### 1. Start Service

Ensure `enable = true` in the configuration file, then start the RobustMQ service:

```bash
./bin/robust-server start
```

After the service starts, you will see logs similar to:
```
Pprof HTTP Server started successfully, listening port: 6777
```

### 2. Generate Flame Graph

Access in your browser:
```
http://127.0.0.1:6777/flamegraph
```

The system will return a performance flame graph in SVG format.

### 3. Analyze Flame Graph

- **Width**: Time proportion of function calls
- **Height**: Call stack depth
- **Color**: Identification markers for different functions
- **Hot spots**: Areas with larger width indicate performance bottlenecks
