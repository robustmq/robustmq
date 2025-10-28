// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package v1alpha1

import (
	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
)

// EDIT THIS FILE!  THIS IS SCAFFOLDING FOR YOU TO OWN!
// NOTE: json tags are required.  Any new fields you add must have json:"-" or json:"fieldName".

// DeploymentMode defines the deployment architecture for RobustMQ
// +kubebuilder:validation:Enum=AllInOne;Microservices
type DeploymentMode string

const (
	// AllInOneMode runs all services (meta, broker, journal) in a single process
	AllInOneMode DeploymentMode = "AllInOne"
	// MicroservicesMode runs each service in separate pods
	MicroservicesMode DeploymentMode = "Microservices"
)

// ServiceRole defines the role of a RobustMQ service
// +kubebuilder:validation:Enum=meta;broker;journal
type ServiceRole string

const (
	MetaRole    ServiceRole = "meta"
	BrokerRole  ServiceRole = "broker"
	JournalRole ServiceRole = "journal"
)

// RobustMQSpec defines the desired state of RobustMQ
type RobustMQSpec struct {
	// INSERT ADDITIONAL SPEC FIELDS - desired state of cluster
	// Important: Run "make" to regenerate code after modifying this file

	// DeploymentMode specifies how to deploy RobustMQ services
	// +kubebuilder:default=AllInOne
	DeploymentMode DeploymentMode `json:"deploymentMode,omitempty"`

	// Image configuration
	Image ImageSpec `json:"image,omitempty"`

	// AllInOne configuration (used when DeploymentMode is AllInOne)
	AllInOne *AllInOneSpec `json:"allInOne,omitempty"`

	// Microservices configuration (used when DeploymentMode is Microservices)
	Microservices *MicroservicesSpec `json:"microservices,omitempty"`

	// Storage configuration
	Storage StorageSpec `json:"storage,omitempty"`

	// Network configuration
	Network NetworkSpec `json:"network,omitempty"`

	// Monitoring configuration
	Monitoring MonitoringSpec `json:"monitoring,omitempty"`

	// Security configuration
	Security SecuritySpec `json:"security,omitempty"`

	// Configuration overrides
	Config ConfigSpec `json:"config,omitempty"`
}

// ImageSpec defines the container image configuration
type ImageSpec struct {
	// Repository is the image repository
	// +kubebuilder:default="robustmq/robustmq"
	Repository string `json:"repository,omitempty"`

	// Tag is the image tag
	// +kubebuilder:default="latest"
	Tag string `json:"tag,omitempty"`

	// PullPolicy is the image pull policy
	// +kubebuilder:default=IfNotPresent
	PullPolicy corev1.PullPolicy `json:"pullPolicy,omitempty"`

	// PullSecrets is a list of secrets for pulling images
	PullSecrets []corev1.LocalObjectReference `json:"pullSecrets,omitempty"`
}

// AllInOneSpec defines configuration for all-in-one deployment
type AllInOneSpec struct {
	// Replicas is the number of replicas for the all-in-one deployment
	// +kubebuilder:default=1
	// +kubebuilder:validation:Minimum=1
	Replicas int32 `json:"replicas,omitempty"`

	// Resources specifies the resource requirements
	Resources corev1.ResourceRequirements `json:"resources,omitempty"`

	// NodeSelector is a selector which must be true for the pod to fit on a node
	NodeSelector map[string]string `json:"nodeSelector,omitempty"`

	// Affinity specifies the pod's scheduling constraints
	Affinity *corev1.Affinity `json:"affinity,omitempty"`

	// Tolerations specifies the pod's tolerations
	Tolerations []corev1.Toleration `json:"tolerations,omitempty"`
}

// MicroservicesSpec defines configuration for microservices deployment
type MicroservicesSpec struct {
	// MetaService configuration
	MetaService ServiceSpec `json:"metaService,omitempty"`

	// BrokerService configuration
	BrokerService ServiceSpec `json:"brokerService,omitempty"`

	// JournalService configuration
	JournalService ServiceSpec `json:"journalService,omitempty"`
}

// ServiceSpec defines configuration for individual services
type ServiceSpec struct {
	// Replicas is the number of replicas for the service
	// +kubebuilder:default=1
	// +kubebuilder:validation:Minimum=1
	Replicas int32 `json:"replicas,omitempty"`

	// Resources specifies the resource requirements
	Resources corev1.ResourceRequirements `json:"resources,omitempty"`

	// NodeSelector is a selector which must be true for the pod to fit on a node
	NodeSelector map[string]string `json:"nodeSelector,omitempty"`

	// Affinity specifies the pod's scheduling constraints
	Affinity *corev1.Affinity `json:"affinity,omitempty"`

	// Tolerations specifies the pod's tolerations
	Tolerations []corev1.Toleration `json:"tolerations,omitempty"`

	// Storage configuration specific to this service
	Storage *ServiceStorageSpec `json:"storage,omitempty"`
}

// StorageSpec defines storage configuration
type StorageSpec struct {
	// StorageClass for persistent volumes
	StorageClass *string `json:"storageClass,omitempty"`

	// Size of the persistent volume
	// +kubebuilder:default="10Gi"
	Size string `json:"size,omitempty"`

	// AccessModes for the persistent volume
	// +kubebuilder:default={"ReadWriteOnce"}
	AccessModes []corev1.PersistentVolumeAccessMode `json:"accessModes,omitempty"`
}

// ServiceStorageSpec defines storage configuration for individual services
type ServiceStorageSpec struct {
	StorageSpec `json:",inline"`

	// Additional storage paths for journal service
	AdditionalPaths []string `json:"additionalPaths,omitempty"`
}

// NetworkSpec defines network configuration
type NetworkSpec struct {
	// MQTT configuration
	MQTT MQTTNetworkSpec `json:"mqtt,omitempty"`

	// Kafka configuration
	Kafka KafkaNetworkSpec `json:"kafka,omitempty"`

	// gRPC configuration
	GRPC GRPCNetworkSpec `json:"grpc,omitempty"`

	// AMQP configuration
	AMQP AMQPNetworkSpec `json:"amqp,omitempty"`
}

// MQTTNetworkSpec defines MQTT network configuration
type MQTTNetworkSpec struct {
	// TCP port for MQTT
	// +kubebuilder:default=1883
	TCPPort int32 `json:"tcpPort,omitempty"`

	// TLS port for MQTT
	// +kubebuilder:default=1885
	TLSPort int32 `json:"tlsPort,omitempty"`

	// WebSocket port for MQTT
	// +kubebuilder:default=8083
	WebSocketPort int32 `json:"webSocketPort,omitempty"`

	// WebSocket TLS port for MQTT
	// +kubebuilder:default=8085
	WebSocketTLSPort int32 `json:"webSocketTLSPort,omitempty"`

	// Service type for MQTT services
	// +kubebuilder:default=ClusterIP
	ServiceType corev1.ServiceType `json:"serviceType,omitempty"`
}

// KafkaNetworkSpec defines Kafka network configuration
type KafkaNetworkSpec struct {
	// Port for Kafka protocol
	// +kubebuilder:default=9092
	Port int32 `json:"port,omitempty"`

	// Service type for Kafka service
	// +kubebuilder:default=ClusterIP
	ServiceType corev1.ServiceType `json:"serviceType,omitempty"`
}

// GRPCNetworkSpec defines gRPC network configuration
type GRPCNetworkSpec struct {
	// Port for gRPC
	// +kubebuilder:default=1228
	Port int32 `json:"port,omitempty"`

	// Service type for gRPC service
	// +kubebuilder:default=ClusterIP
	ServiceType corev1.ServiceType `json:"serviceType,omitempty"`
}

// AMQPNetworkSpec defines AMQP network configuration
type AMQPNetworkSpec struct {
	// Port for AMQP
	// +kubebuilder:default=5672
	Port int32 `json:"port,omitempty"`

	// Service type for AMQP service
	// +kubebuilder:default=ClusterIP
	ServiceType corev1.ServiceType `json:"serviceType,omitempty"`
}

// MonitoringSpec defines monitoring configuration
type MonitoringSpec struct {
	// Enable Prometheus monitoring
	// +kubebuilder:default=true
	Enabled bool `json:"enabled,omitempty"`

	// Prometheus configuration
	Prometheus PrometheusSpec `json:"prometheus,omitempty"`

	// ServiceMonitor configuration
	ServiceMonitor ServiceMonitorSpec `json:"serviceMonitor,omitempty"`
}

// PrometheusSpec defines Prometheus configuration
type PrometheusSpec struct {
	// Enable Prometheus metrics collection
	// +kubebuilder:default=true
	Enable bool `json:"enable,omitempty"`

	// Prometheus port
	// +kubebuilder:default=9091
	Port int32 `json:"port,omitempty"`

	// Prometheus model (pull/push)
	// +kubebuilder:default="pull"
	Model string `json:"model,omitempty"`
}

// ServiceMonitorSpec defines ServiceMonitor configuration
type ServiceMonitorSpec struct {
	// Enable ServiceMonitor creation
	// +kubebuilder:default=true
	Enabled bool `json:"enabled,omitempty"`

	// Additional labels for ServiceMonitor
	Labels map[string]string `json:"labels,omitempty"`

	// Scrape interval
	// +kubebuilder:default="30s"
	Interval string `json:"interval,omitempty"`
}

// SecuritySpec defines security configuration
type SecuritySpec struct {
	// TLS configuration
	TLS TLSSpec `json:"tls,omitempty"`

	// Authentication configuration
	Auth AuthSpec `json:"auth,omitempty"`
}

// TLSSpec defines TLS configuration
type TLSSpec struct {
	// Enable TLS
	Enabled bool `json:"enabled,omitempty"`

	// Secret name containing TLS certificates
	SecretName string `json:"secretName,omitempty"`

	// CA certificate configuration
	CA CASpec `json:"ca,omitempty"`
}

// CASpec defines CA certificate configuration
type CASpec struct {
	// Secret name containing CA certificate
	SecretName string `json:"secretName,omitempty"`

	// Auto-generate CA certificate
	AutoGenerate bool `json:"autoGenerate,omitempty"`
}

// AuthSpec defines authentication configuration
type AuthSpec struct {
	// Default username
	// +kubebuilder:default="admin"
	DefaultUser string `json:"defaultUser,omitempty"`

	// Secret name containing default password
	PasswordSecret string `json:"passwordSecret,omitempty"`

	// Storage type for authentication data
	// +kubebuilder:default="placement"
	StorageType string `json:"storageType,omitempty"`
}

// ConfigSpec defines configuration overrides
type ConfigSpec struct {
	// Additional configuration as key-value pairs
	Additional map[string]string `json:"additional,omitempty"`

	// ConfigMap name containing additional configuration
	ConfigMapName string `json:"configMapName,omitempty"`
}

// RobustMQStatus defines the observed state of RobustMQ
type RobustMQStatus struct {
	// INSERT ADDITIONAL STATUS FIELD - define observed state of cluster
	// Important: Run "make" to regenerate code after modifying this file

	// Phase represents the current phase of the RobustMQ deployment
	Phase string `json:"phase,omitempty"`

	// Conditions represent the latest available observations of an object's state
	Conditions []metav1.Condition `json:"conditions,omitempty"`

	// DeploymentStatuses represent the status of individual deployments
	DeploymentStatuses map[string]DeploymentStatus `json:"deploymentStatuses,omitempty"`

	// ServiceStatuses represent the status of services
	ServiceStatuses map[string]ServiceStatus `json:"serviceStatuses,omitempty"`

	// ClusterInfo contains information about the RobustMQ cluster
	ClusterInfo ClusterInfo `json:"clusterInfo,omitempty"`
}

// DeploymentStatus represents the status of a deployment
type DeploymentStatus struct {
	// Replicas is the number of desired replicas
	Replicas int32 `json:"replicas,omitempty"`

	// ReadyReplicas is the number of ready replicas
	ReadyReplicas int32 `json:"readyReplicas,omitempty"`

	// AvailableReplicas is the number of available replicas
	AvailableReplicas int32 `json:"availableReplicas,omitempty"`
}

// ServiceStatus represents the status of a service
type ServiceStatus struct {
	// ClusterIP is the cluster IP address of the service
	ClusterIP string `json:"clusterIP,omitempty"`

	// ExternalIPs is a list of external IP addresses
	ExternalIPs []string `json:"externalIPs,omitempty"`

	// LoadBalancerIngress represents the status of load balancer ingress
	LoadBalancerIngress []corev1.LoadBalancerIngress `json:"loadBalancerIngress,omitempty"`
}

// ClusterInfo contains information about the RobustMQ cluster
type ClusterInfo struct {
	// ClusterName is the name of the RobustMQ cluster
	ClusterName string `json:"clusterName,omitempty"`

	// Nodes is a list of cluster nodes
	Nodes []NodeInfo `json:"nodes,omitempty"`

	// MetaLeader is the current meta service leader
	MetaLeader string `json:"metaLeader,omitempty"`
}

// NodeInfo contains information about a cluster node
type NodeInfo struct {
	// ID is the node ID
	ID string `json:"id,omitempty"`

	// Roles is a list of roles this node serves
	Roles []ServiceRole `json:"roles,omitempty"`

	// Address is the network address of the node
	Address string `json:"address,omitempty"`

	// Status is the current status of the node
	Status string `json:"status,omitempty"`
}

//+kubebuilder:object:root=true
//+kubebuilder:subresource:status
//+kubebuilder:printcolumn:name="Mode",type=string,JSONPath=`.spec.deploymentMode`
//+kubebuilder:printcolumn:name="Phase",type=string,JSONPath=`.status.phase`
//+kubebuilder:printcolumn:name="Cluster",type=string,JSONPath=`.status.clusterInfo.clusterName`
//+kubebuilder:printcolumn:name="Age",type=date,JSONPath=`.metadata.creationTimestamp`

// RobustMQ is the Schema for the robustmqs API
type RobustMQ struct {
	metav1.TypeMeta   `json:",inline"`
	metav1.ObjectMeta `json:"metadata,omitempty"`

	Spec   RobustMQSpec   `json:"spec,omitempty"`
	Status RobustMQStatus `json:"status,omitempty"`
}

//+kubebuilder:object:root=true

// RobustMQList contains a list of RobustMQ
type RobustMQList struct {
	metav1.TypeMeta `json:",inline"`
	metav1.ListMeta `json:"metadata,omitempty"`
	Items           []RobustMQ `json:"items"`
}

func init() {
	SchemeBuilder.Register(&RobustMQ{}, &RobustMQList{})
}
