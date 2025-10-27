# Kubernetes Operator Deployment

This guide describes how to deploy and manage RobustMQ clusters using RobustMQ Operator in Kubernetes environments.

## Overview

RobustMQ Operator is a Kubernetes controller that automates the deployment, scaling, and management of RobustMQ clusters. It provides a declarative API to manage RobustMQ instances, supporting multiple deployment modes and advanced features.

### Key Features

- **Deployment Modes**: Support for AllInOne and Microservices deployment modes
- **Auto-scaling**: Automatic management of RobustMQ component replicas
- **Configuration Management**: Automatic generation and management of RobustMQ configurations
- **Multi-protocol Support**: MQTT, Kafka, gRPC, and AMQP protocol support
- **Monitoring Integration**: Built-in Prometheus monitoring and ServiceMonitor support
- **Security Features**: TLS encryption and authentication support
- **Storage Management**: Persistent volume management for data storage
- **High Availability**: Support for multi-replica deployments

## Prerequisites

### System Requirements

- **Kubernetes**: Version 1.20+
- **kubectl**: Configured to access your cluster
- **Helm**: Version 3.x (optional, for Helm-based installation)

### Check Cluster Status

```bash
# Check cluster connection
kubectl cluster-info

# Check node status
kubectl get nodes

# Check storage classes
kubectl get storageclass
```

## Installing Operator

### Method 1: Using Deployment Script (Recommended)

```bash
# Clone project
git clone https://github.com/robustmq/robustmq.git
cd robustmq/operator

# Deploy with default configuration
./deploy.sh

# Or specify custom image
IMAGE=your-registry/robustmq-operator:v1.0.0 ./deploy.sh
```

### Method 2: Manual Deployment

```bash
# Create namespace
kubectl create namespace robustmq-system

# Deploy Operator
kubectl apply -f robustmq.yaml

# Wait for Operator to be ready
kubectl wait --for=condition=available --timeout=300s \
  deployment/robustmq-operator-controller-manager -n robustmq-system
```

### Verify Installation

```bash
# Check Operator status
kubectl get pods -n robustmq-system

# Check CRD
kubectl get crd | grep robustmq

# View Operator logs
kubectl logs -n robustmq-system deployment/robustmq-operator-controller-manager -f
```

## Deploying RobustMQ Instances

### AllInOne Deployment (Development/Testing)

AllInOne mode runs all services (meta, broker, journal) in a single StatefulSet, suitable for development and testing environments.

```yaml
apiVersion: robustmq.io/v1alpha1
kind: RobustMQ
metadata:
  name: robustmq-dev
  namespace: default
spec:
  # Deployment mode
  deploymentMode: AllInOne
  
  # Image configuration
  image:
    repository: robustmq/robustmq
    tag: latest
    pullPolicy: IfNotPresent
  
  # AllInOne configuration
  allInOne:
    replicas: 1
    resources:
      requests:
        memory: "1Gi"
        cpu: "500m"
      limits:
        memory: "2Gi"
        cpu: "1000m"
  
  # Storage configuration
  storage:
    storageClass: standard
    size: "10Gi"
    accessModes:
      - ReadWriteOnce
  
  # Network configuration
  network:
    mqtt:
      tcpPort: 1883
      tlsPort: 1885
      webSocketPort: 8083
      webSocketTLSPort: 8084
      serviceType: ClusterIP
  
  # Monitoring configuration
  monitoring:
    enabled: true
    prometheus:
      enable: true
      port: 9091
    serviceMonitor:
      enabled: true
      interval: "30s"
```

### Microservices Deployment (Production)

Microservices mode runs each service in independent StatefulSets, providing better scalability and isolation.

```yaml
apiVersion: robustmq.io/v1alpha1
kind: RobustMQ
metadata:
  name: robustmq-prod
  namespace: default
spec:
  # Microservices deployment mode
  deploymentMode: Microservices
  
  # Image configuration
  image:
    repository: robustmq/robustmq
    tag: latest
    pullPolicy: IfNotPresent
  
  # Microservices configuration
  microservices:
    metaService:
      replicas: 3
      resources:
        requests:
          memory: "512Mi"
          cpu: "250m"
        limits:
          memory: "1Gi"
          cpu: "500m"
    brokerService:
      replicas: 2
      resources:
        requests:
          memory: "1Gi"
          cpu: "500m"
        limits:
          memory: "2Gi"
          cpu: "1000m"
    journalService:
      replicas: 3
      resources:
        requests:
          memory: "1Gi"
          cpu: "500m"
        limits:
          memory: "2Gi"
          cpu: "1000m"
      storage:
        size: "50Gi"
  
  # Storage configuration
  storage:
    storageClass: fast-ssd
    size: "20Gi"
    accessModes:
      - ReadWriteOnce
  
  # Network configuration
  network:
    mqtt:
      serviceType: LoadBalancer
    kafka:
      serviceType: LoadBalancer
    grpc:
      serviceType: ClusterIP
  
  # Monitoring configuration
  monitoring:
    enabled: true
    serviceMonitor:
      enabled: true
      labels:
        prometheus: "kube-prometheus"
  
  # Security configuration
  security:
    tls:
      enabled: true
      secretName: "robustmq-tls"
      ca:
        autoGenerate: true
    auth:
      defaultUser: "admin"
      passwordSecret: "robustmq-auth"
```

### Deploy Instance

```bash
# Deploy AllInOne instance
kubectl apply -f sample-robustmq.yaml

# Or deploy custom configuration
kubectl apply -f your-robustmq-config.yaml

# Check deployment status
kubectl get robustmq

# View detailed information
kubectl describe robustmq robustmq-sample
```

## Configuration Reference

### Deployment Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| AllInOne | All services run in a single StatefulSet | Development, testing, small-scale deployment |
| Microservices | Each service runs in independent StatefulSets | Production environment, large-scale deployment |

### Image Configuration

```yaml
image:
  repository: robustmq/robustmq  # Image repository
  tag: latest                    # Image tag
  pullPolicy: IfNotPresent       # Pull policy
  pullSecrets:                   # Pull secrets
    - name: my-registry-secret
```

### Storage Configuration

```yaml
storage:
  storageClass: fast-ssd         # Storage class
  size: "20Gi"                   # Storage size
  accessModes:                   # Access modes
    - ReadWriteOnce
```

### Network Configuration

```yaml
network:
  mqtt:
    tcpPort: 1883                # MQTT TCP port
    tlsPort: 1885                # MQTT TLS port
    webSocketPort: 8083          # WebSocket port
    webSocketTLSPort: 8084       # WebSocket TLS port
    serviceType: LoadBalancer    # Service type
  kafka:
    port: 9092                   # Kafka port
    serviceType: LoadBalancer
  grpc:
    port: 1228                   # gRPC port
    serviceType: ClusterIP
  amqp:
    port: 5672                   # AMQP port
    serviceType: ClusterIP
```

### Monitoring Configuration

```yaml
monitoring:
  enabled: true                  # Enable monitoring
  prometheus:
    enable: true                 # Enable Prometheus metrics
    port: 9091                   # Metrics port
    model: "pull"                # Pull mode
  serviceMonitor:
    enabled: true                # Enable ServiceMonitor
    interval: "30s"              # Scrape interval
    labels:                      # Additional labels
      prometheus: "kube-prometheus"
```

### Security Configuration

```yaml
security:
  tls:
    enabled: true                # Enable TLS
    secretName: "robustmq-tls"   # TLS secret name
    ca:
      autoGenerate: true         # Auto-generate CA
  auth:
    defaultUser: "admin"         # Default username
    passwordSecret: "robustmq-auth"  # Password secret
    storageType: "placement"     # Storage type
```

## Management Operations

### View Status

```bash
# View all RobustMQ instances
kubectl get robustmq

# View specific instance status
kubectl get robustmq robustmq-sample -o yaml

# View Pod status
kubectl get pods -l app=robustmq

# View service status
kubectl get services -l app=robustmq
```

### Scaling

```bash
# Edit configuration for scaling
kubectl edit robustmq robustmq-sample

# Or use patch command
kubectl patch robustmq robustmq-sample --type='merge' \
  -p='{"spec":{"allInOne":{"replicas":3}}}'
```

### Update Configuration

```bash
# Update image version
kubectl patch robustmq robustmq-sample --type='merge' \
  -p='{"spec":{"image":{"tag":"v1.1.0"}}}'

# Update resource configuration
kubectl patch robustmq robustmq-sample --type='merge' \
  -p='{"spec":{"allInOne":{"resources":{"limits":{"memory":"4Gi"}}}}}'
```

### Delete Instance

```bash
# Delete specific instance
kubectl delete robustmq robustmq-sample

# Delete all instances
kubectl delete robustmq --all
```

## Monitoring and Logs

### View Logs

```bash
# View Operator logs
kubectl logs -n robustmq-system deployment/robustmq-operator-controller-manager -f

# View RobustMQ Pod logs
kubectl logs -l app=robustmq -f

# View specific Pod logs
kubectl logs robustmq-sample-0 -f
```

### Monitoring Metrics

```bash
# Check ServiceMonitor
kubectl get servicemonitor

# Check Prometheus metrics endpoint
kubectl port-forward svc/robustmq-sample-mqtt 9091:9091
curl http://localhost:9091/metrics
```

### Health Checks

```bash
# Check Pod health status
kubectl get pods -l app=robustmq

# Check service endpoints
kubectl get endpoints -l app=robustmq

# Check events
kubectl get events --sort-by=.metadata.creationTimestamp
```

## Troubleshooting

### Common Issues

1. **Operator Not Starting**

   ```bash
   # Check RBAC permissions
   kubectl auth can-i create statefulsets --as=system:serviceaccount:robustmq-system:robustmq-operator-controller-manager
   
   # Check Operator logs
   kubectl logs -n robustmq-system deployment/robustmq-operator-controller-manager
   ```

2. **Pod Startup Failure**

   ```bash
   # Check resource limits
   kubectl describe pod robustmq-sample-0
   
   # Check storage configuration
   kubectl get pvc
   
   # Check image pull
   kubectl describe pod robustmq-sample-0 | grep -A 5 "Events:"
   ```

3. **Service Not Accessible**

   ```bash
   # Check service configuration
   kubectl get svc -l app=robustmq
   
   # Check network policies
   kubectl get networkpolicy
   
   # Test service connection
   kubectl run test-pod --image=busybox --rm -it -- nslookup robustmq-sample-mqtt
   ```

### Debug Commands

```bash
# Enter Pod for debugging
kubectl exec -it robustmq-sample-0 -- /bin/bash

# Check configuration
kubectl exec robustmq-sample-0 -- cat /robustmq/config/server.toml

# Check cluster status
kubectl exec robustmq-sample-0 -- ./libs/cli-command cluster info

# Check resource usage
kubectl top pods -l app=robustmq
```

## Uninstalling Operator

### Using Uninstall Script

```bash
# Uninstall with default configuration
./undeploy.sh

# Or specify namespace
NAMESPACE=robustmq-system ./undeploy.sh
```

### Manual Uninstall

```bash
# Delete all RobustMQ instances
kubectl delete robustmq --all --all-namespaces

# Delete Operator
kubectl delete -f robustmq.yaml

# Delete namespace (optional)
kubectl delete namespace robustmq-system
```

## Best Practices

### Production Environment Recommendations

1. **Resource Planning**
   - Set appropriate resource requests and limits for each component
   - Use QoS classes to provide resource guarantees for critical components
   - Monitor resource usage and adjust regularly

2. **Storage Configuration**
   - Use high-performance storage classes (such as SSD)
   - Configure independent storage volumes for different services
   - Regularly backup important data

3. **Network Configuration**
   - Use LoadBalancer or Ingress to expose services
   - Configure network policies to restrict access
   - Enable TLS encryption

4. **Monitoring and Alerting**
   - Configure Prometheus monitoring
   - Set up alerts for key metrics
   - Regularly check cluster health status

5. **Security Configuration**
   - Enable TLS encryption
   - Configure authentication
   - Use RBAC to control access permissions

### Development Environment Recommendations

1. **Use AllInOne mode** to simplify deployment
2. **Set smaller resource limits** to save costs
3. **Use ClusterIP service type** to reduce configuration complexity
4. **Enable detailed logging** for easier debugging
