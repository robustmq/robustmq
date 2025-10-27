# RobustMQ Monitoring Configuration

This directory contains monitoring configurations for RobustMQ using Prometheus and Grafana.

## Files Structure

```
monitoring/
├── prometheus.yml                    # Prometheus configuration
├── grafana/
│   ├── datasources/
│   │   └── prometheus.yml           # Grafana Prometheus datasource
│   └── dashboards/
│       ├── dashboard.yml            # Dashboard provisioning config
│       └── robustmq-dashboard.json  # RobustMQ dashboard definition
└── README.md                        # This file
```

## Usage

### Start with Monitoring

To start RobustMQ with monitoring stack:

```bash
# Start all services including monitoring
docker-compose --profile monitoring up -d

# Or start only monitoring services
docker-compose --profile monitoring up -d prometheus grafana
```

### Access URLs

- **RobustMQ Admin**: http://localhost:8080
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)

### Grafana Configuration

The Grafana instance is pre-configured with:

1. **Prometheus Datasource**: Automatically configured to connect to Prometheus
2. **RobustMQ Dashboard**: Pre-built dashboard with key metrics
3. **Auto-provisioning**: Dashboards and datasources are automatically loaded

### Metrics Collected

The monitoring stack collects the following metrics:

- **HTTP Metrics**: Request rate, duration, status codes
- **MQTT Metrics**: Active connections, message throughput
- **System Metrics**: CPU, memory, disk usage (if node-exporter is available)
- **Custom Metrics**: RobustMQ-specific business metrics

### Customization

#### Adding New Dashboards

1. Create a new JSON file in `grafana/dashboards/`
2. The dashboard will be automatically loaded by Grafana

#### Modifying Prometheus Configuration

Edit `prometheus.yml` to:
- Add new scrape targets
- Modify scrape intervals
- Add alerting rules
- Configure remote storage

#### Adding Alerting Rules

1. Create a new YAML file with alerting rules
2. Add it to the `rule_files` section in `prometheus.yml`
3. Configure Alertmanager for notifications

## Troubleshooting

### Grafana Not Loading Dashboards

1. Check that the dashboard files are in the correct directory
2. Verify file permissions
3. Check Grafana logs: `docker logs robustmq-grafana`

### Prometheus Not Scraping Metrics

1. Verify the target URLs are correct
2. Check that RobustMQ is exposing metrics on the expected endpoints
3. Check Prometheus logs: `docker logs robustmq-prometheus`

### Missing Metrics

1. Ensure RobustMQ is configured to expose metrics
2. Check that the metrics endpoints are accessible
3. Verify the scrape configuration in `prometheus.yml`
