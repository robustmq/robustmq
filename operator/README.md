# RobustMQ Operator

RobustMQ Operator is a Kubernetes operator for managing RobustMQ clusters. It automates the deployment, scaling, and management of RobustMQ instances in Kubernetes environments.

## Features

- **Deployment Modes**: Support both AllInOne and Microservices deployment modes
- **Auto-scaling**: Automatic scaling of RobustMQ components
- **Configuration Management**: Automatic generation and management of RobustMQ configurations
- **Multi-protocol Support**: MQTT, Kafka, gRPC, and AMQP protocol support
- **Monitoring Integration**: Built-in Prometheus monitoring and ServiceMonitor support
- **Security**: TLS encryption and authentication support
- **Storage Management**: Persistent volume management for data storage
- **High Availability**: Support for multi-replica deployments

## Architecture

RobustMQ Operator manages the following components:

- **Meta Service**: Cluster metadata and coordination service
- **Broker Service**: Message broker for handling client connections
- **Journal Service**: Persistent message storage service

### Deployment Modes

#### AllInOne Mode
All services (meta, broker, journal) run in a single StatefulSet, suitable for development and small-scale deployments.

#### Microservices Mode
Each service runs in separate StatefulSets, providing better scalability and isolation for production environments.

## Quick Start

### Prerequisites

- Kubernetes cluster (v1.20+)
- kubectl configured to access your cluster
- Helm 3.x (optional, for Helm-based installation)

### Installation

1. **Install the CRD and Operator**:

```bash
kubectl apply -f robustmq.yaml
```

2. **Create a RobustMQ instance**:

```bash
kubectl apply -f sample-robustmq.yaml
```

3. **Check the status**:

```bash
kubectl get robustmq
kubectl describe robustmq robustmq-sample
```

### Configuration Examples

#### AllInOne Deployment

```yaml
apiVersion: robustmq.io/v1alpha1
kind: RobustMQ
metadata:
  name: robustmq-allinone
  namespace: default
spec:
  deploymentMode: AllInOne
  image:
    repository: robustmq/robustmq
    tag: latest
  allInOne:
    replicas: 1
    resources:
      requests:
        memory: "1Gi"
        cpu: "500m"
      limits:
        memory: "2Gi"
        cpu: "1000m"
  storage:
    size: "10Gi"
  monitoring:
    enabled: true
```

#### Microservices Deployment

```yaml
apiVersion: robustmq.io/v1alpha1
kind: RobustMQ
metadata:
  name: robustmq-microservices
  namespace: default
spec:
  deploymentMode: Microservices
  image:
    repository: robustmq/robustmq
    tag: latest
  microservices:
    metaService:
      replicas: 3
    brokerService:
      replicas: 2
    journalService:
      replicas: 3
  storage:
    size: "20Gi"
  network:
    mqtt:
      serviceType: LoadBalancer
```

## API Reference

### RobustMQSpec

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `deploymentMode` | string | Deployment mode: AllInOne or Microservices | AllInOne |
| `image` | ImageSpec | Container image configuration | - |
| `allInOne` | AllInOneSpec | AllInOne deployment configuration | - |
| `microservices` | MicroservicesSpec | Microservices deployment configuration | - |
| `storage` | StorageSpec | Storage configuration | - |
| `network` | NetworkSpec | Network configuration | - |
| `monitoring` | MonitoringSpec | Monitoring configuration | - |
| `security` | SecuritySpec | Security configuration | - |
| `config` | ConfigSpec | Additional configuration | - |

### ImageSpec

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `repository` | string | Image repository | robustmq/robustmq |
| `tag` | string | Image tag | latest |
| `pullPolicy` | string | Image pull policy | IfNotPresent |
| `pullSecrets` | []LocalObjectReference | Image pull secrets | - |

### NetworkSpec

| Field | Type | Description |
|-------|------|-------------|
| `mqtt` | MQTTNetworkSpec | MQTT network configuration |
| `kafka` | KafkaNetworkSpec | Kafka network configuration |
| `grpc` | GRPCNetworkSpec | gRPC network configuration |
| `amqp` | AMQPNetworkSpec | AMQP network configuration |

### MQTTNetworkSpec

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `tcpPort` | int32 | TCP port for MQTT | 1883 |
| `tlsPort` | int32 | TLS port for MQTT | 1885 |
| `webSocketPort` | int32 | WebSocket port | 8083 |
| `webSocketTLSPort` | int32 | WebSocket TLS port | 8085 |
| `serviceType` | ServiceType | Kubernetes service type | ClusterIP |

## Development

### Building from Source

1. **Clone the repository**:

```bash
git clone https://github.com/robustmq/robustmq.git
cd robustmq/operator
```

2. **Install dependencies**:

```bash
go mod download
```

3. **Build the operator**:

```bash
make build
```

4. **Run tests**:

```bash
make test
```

5. **Build Docker image**:

```bash
make docker-build IMG=your-registry/robustmq-operator:tag
```

### Running Locally

You can run the operator locally against a Kubernetes cluster:

```bash
make install  # Install CRDs
make run      # Run the operator locally
```

### Testing

Run the test suite:

```bash
make test
```

Run integration tests:

```bash
make test-integration
```

## Monitoring

The operator supports Prometheus monitoring out of the box. When monitoring is enabled, it creates:

- Prometheus metrics endpoints on each component
- ServiceMonitor resources for automatic discovery
- Grafana dashboards (if using Grafana operator)

### Metrics

The operator exposes the following metrics:

- `robustmq_cluster_nodes_total`: Total number of cluster nodes
- `robustmq_cluster_status`: Cluster status (0=unhealthy, 1=healthy)
- `robustmq_component_replicas`: Number of replicas per component
- `robustmq_component_ready_replicas`: Number of ready replicas per component

## Security

### TLS Configuration

Enable TLS encryption:

```yaml
spec:
  security:
    tls:
      enabled: true
      secretName: robustmq-tls
      ca:
        autoGenerate: true
```

### Authentication

Configure authentication:

```yaml
spec:
  security:
    auth:
      defaultUser: admin
      passwordSecret: robustmq-auth
      storageType: placement
```

## Troubleshooting

### Common Issues

1. **Operator not starting**: Check operator logs and RBAC permissions
2. **RobustMQ pods failing**: Check resource limits and storage configuration
3. **Services not accessible**: Verify network configuration and service types

### Debug Commands

```bash
# Check operator logs
kubectl logs -n robustmq-system deployment/robustmq-operator-controller-manager

# Check RobustMQ status
kubectl get robustmq -o yaml

# Check pod status
kubectl get pods -l app=robustmq

# Check service status
kubectl get services -l app=robustmq
```

## Contributing

We welcome contributions! Please see our [Contributing Guide](../CONTRIBUTING.md) for details.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](../LICENSE) file for details.

## Support

- GitHub Issues: [https://github.com/robustmq/robustmq/issues](https://github.com/robustmq/robustmq/issues)
- Documentation: [https://robustmq.com/docs](https://robustmq.com/docs)
- Community: [https://github.com/robustmq/robustmq/discussions](https://github.com/robustmq/robustmq/discussions)