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

package controllers

import (
	"context"
	"fmt"
	"time"

	appsv1 "k8s.io/api/apps/v1"
	corev1 "k8s.io/api/core/v1"
	"k8s.io/apimachinery/pkg/api/errors"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/types"
	ctrl "sigs.k8s.io/controller-runtime"
	"sigs.k8s.io/controller-runtime/pkg/client"
	"sigs.k8s.io/controller-runtime/pkg/controller/controllerutil"
	"sigs.k8s.io/controller-runtime/pkg/log"

	robustmqv1alpha1 "github.com/robustmq/robustmq-operator/api/v1alpha1"
)

// RobustMQReconciler reconciles a RobustMQ object
type RobustMQReconciler struct {
	client.Client
	Scheme *runtime.Scheme
}

// Phase constants
const (
	PhaseInitializing = "Initializing"
	PhaseRunning      = "Running"
	PhaseFailed       = "Failed"
	PhaseUpdating     = "Updating"
)

// Condition types
const (
	ConditionTypeReady             = "Ready"
	ConditionTypeConfigMapReady    = "ConfigMapReady"
	ConditionTypeServiceReady      = "ServiceReady"
	ConditionTypeDeploymentReady   = "DeploymentReady"
	ConditionTypeStatefulSetReady  = "StatefulSetReady"
)

//+kubebuilder:rbac:groups=robustmq.io,resources=robustmqs,verbs=get;list;watch;create;update;patch;delete
//+kubebuilder:rbac:groups=robustmq.io,resources=robustmqs/status,verbs=get;update;patch
//+kubebuilder:rbac:groups=robustmq.io,resources=robustmqs/finalizers,verbs=update
//+kubebuilder:rbac:groups=apps,resources=deployments,verbs=get;list;watch;create;update;patch;delete
//+kubebuilder:rbac:groups=apps,resources=statefulsets,verbs=get;list;watch;create;update;patch;delete
//+kubebuilder:rbac:groups="",resources=services,verbs=get;list;watch;create;update;patch;delete
//+kubebuilder:rbac:groups="",resources=configmaps,verbs=get;list;watch;create;update;patch;delete
//+kubebuilder:rbac:groups="",resources=persistentvolumeclaims,verbs=get;list;watch;create;update;patch;delete
//+kubebuilder:rbac:groups="",resources=secrets,verbs=get;list;watch;create;update;patch;delete
//+kubebuilder:rbac:groups="",resources=pods,verbs=get;list;watch

// Reconcile is part of the main kubernetes reconciliation loop which aims to
// move the current state of the cluster closer to the desired state.
func (r *RobustMQReconciler) Reconcile(ctx context.Context, req ctrl.Request) (ctrl.Result, error) {
	logger := log.FromContext(ctx)

	// Fetch the RobustMQ instance
	robustmq := &robustmqv1alpha1.RobustMQ{}
	err := r.Get(ctx, req.NamespacedName, robustmq)
	if err != nil {
		if errors.IsNotFound(err) {
			// Request object not found, could have been deleted after reconcile request.
			logger.Info("RobustMQ resource not found. Ignoring since object must be deleted")
			return ctrl.Result{}, nil
		}
		// Error reading the object - requeue the request.
		logger.Error(err, "Failed to get RobustMQ")
		return ctrl.Result{}, err
	}

	// Initialize status if needed
	if robustmq.Status.Phase == "" {
		robustmq.Status.Phase = PhaseInitializing
		if err := r.Status().Update(ctx, robustmq); err != nil {
			logger.Error(err, "Failed to update RobustMQ status")
			return ctrl.Result{}, err
		}
	}

	// Reconcile ConfigMap
	if err := r.reconcileConfigMap(ctx, robustmq); err != nil {
		logger.Error(err, "Failed to reconcile ConfigMap")
		return ctrl.Result{}, err
	}

	// Reconcile Services
	if err := r.reconcileServices(ctx, robustmq); err != nil {
		logger.Error(err, "Failed to reconcile Services")
		return ctrl.Result{}, err
	}

	// Reconcile based on deployment mode
	switch robustmq.Spec.DeploymentMode {
	case robustmqv1alpha1.AllInOneMode:
		if err := r.reconcileAllInOne(ctx, robustmq); err != nil {
			logger.Error(err, "Failed to reconcile AllInOne deployment")
			return ctrl.Result{}, err
		}
	case robustmqv1alpha1.MicroservicesMode:
		if err := r.reconcileMicroservices(ctx, robustmq); err != nil {
			logger.Error(err, "Failed to reconcile Microservices deployment")
			return ctrl.Result{}, err
		}
	default:
		// Default to AllInOne
		if err := r.reconcileAllInOne(ctx, robustmq); err != nil {
			logger.Error(err, "Failed to reconcile default AllInOne deployment")
			return ctrl.Result{}, err
		}
	}

	// Update status
	if err := r.updateStatus(ctx, robustmq); err != nil {
		logger.Error(err, "Failed to update status")
		return ctrl.Result{}, err
	}

	return ctrl.Result{RequeueAfter: time.Minute * 5}, nil
}

// reconcileConfigMap creates or updates the ConfigMap for RobustMQ configuration
func (r *RobustMQReconciler) reconcileConfigMap(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	logger := log.FromContext(ctx)

	configMap := &corev1.ConfigMap{
		ObjectMeta: metav1.ObjectMeta{
			Name:      robustmq.Name + "-config",
			Namespace: robustmq.Namespace,
		},
	}

	// Set RobustMQ instance as the owner and controller
	if err := controllerutil.SetControllerReference(robustmq, configMap, r.Scheme); err != nil {
		return err
	}

	// Define the desired state
	configMap.Data = r.generateConfigData(robustmq)

	// Create or update the ConfigMap
	found := &corev1.ConfigMap{}
	err := r.Get(ctx, types.NamespacedName{Name: configMap.Name, Namespace: configMap.Namespace}, found)
	if err != nil && errors.IsNotFound(err) {
		logger.Info("Creating a new ConfigMap", "ConfigMap.Namespace", configMap.Namespace, "ConfigMap.Name", configMap.Name)
		if err := r.Create(ctx, configMap); err != nil {
			return err
		}
	} else if err != nil {
		return err
	} else {
		// Update existing ConfigMap
		found.Data = configMap.Data
		logger.Info("Updating ConfigMap", "ConfigMap.Namespace", found.Namespace, "ConfigMap.Name", found.Name)
		if err := r.Update(ctx, found); err != nil {
			return err
		}
	}

	return nil
}

// generateConfigData generates configuration data for RobustMQ
func (r *RobustMQReconciler) generateConfigData(robustmq *robustmqv1alpha1.RobustMQ) map[string]string {
	config := map[string]string{
		"server.toml": r.generateServerConfig(robustmq),
	}

	// Add tracing configuration
	config["server-tracing.toml"] = r.generateTracingConfig(robustmq)

	// Add any additional configuration from spec
	for key, value := range robustmq.Spec.Config.Additional {
		config[key] = value
	}

	return config
}

// generateServerConfig generates the main server configuration
func (r *RobustMQReconciler) generateServerConfig(robustmq *robustmqv1alpha1.RobustMQ) string {
	clusterName := robustmq.Name
	if robustmq.Status.ClusterInfo.ClusterName != "" {
		clusterName = robustmq.Status.ClusterInfo.ClusterName
	}

	// Generate basic configuration
	config := fmt.Sprintf(`# RobustMQ Configuration
cluster_name = "%s"
broker_id = 1
grpc_port = %d

`, clusterName, robustmq.Spec.Network.GRPC.Port)

	// Add roles based on deployment mode
	switch robustmq.Spec.DeploymentMode {
	case robustmqv1alpha1.AllInOneMode:
		config += `roles = ["meta", "broker", "journal"]
`
	case robustmqv1alpha1.MicroservicesMode:
		// This will be overridden per service
		config += `roles = ["meta"]
`
	}

	// Add meta service configuration
	config += fmt.Sprintf(`meta_service = { 1 = "%s-meta-service:%d" }

`, robustmq.Name, robustmq.Spec.Network.GRPC.Port)

	// Add Prometheus configuration if monitoring is enabled
	if robustmq.Spec.Monitoring.Enabled {
		config += fmt.Sprintf(`[prometheus]
enable = %t
model = "%s"
port = %d
interval = 10

`, robustmq.Spec.Monitoring.Prometheus.Enable, robustmq.Spec.Monitoring.Prometheus.Model, robustmq.Spec.Monitoring.Prometheus.Port)
	}

	// Add log configuration
	config += `[log]
log_config = "/robustmq/config/logger.toml"
log_path = "/robustmq/logs"

`

	// Add RocksDB configuration
	config += `[rocksdb]
data_path = "/robustmq/data"
max_open_files = 10000

`

	// Add MQTT configuration
	config += fmt.Sprintf(`[mqtt.server]
tcp_port = %d
tls_port = %d
websocket_port = %d
websockets_port = %d

[mqtt.auth.storage]
storage_type = "%s"

[mqtt.message.storage]
storage_type = "memory"

[mqtt.runtime]
heartbeat_timeout = "10s"
default_user = "%s"
default_password = "pwd123"
max_connection_num = 5000000

`, robustmq.Spec.Network.MQTT.TCPPort, robustmq.Spec.Network.MQTT.TLSPort,
		robustmq.Spec.Network.MQTT.WebSocketPort, robustmq.Spec.Network.MQTT.WebSocketTLSPort,
		robustmq.Spec.Security.Auth.StorageType, robustmq.Spec.Security.Auth.DefaultUser)

	// Add journal configuration
	config += `[journal.server]
tcp_port = 1771

[journal.storage]
data_paths = ["/robustmq/journal-data"]
rocksdb_max_open_files = 10000

[journal.runtime]
enable_auto_create_shard = false
shard_replica_num = 1
max_segment_size = 1048576
`

	return config
}

// generateTracingConfig generates the tracing configuration
func (r *RobustMQReconciler) generateTracingConfig(robustmq *robustmqv1alpha1.RobustMQ) string {
	return `# RobustMQ Tracing Configuration
[stdout]
kind = "console"
level = "info"

[server]
kind = "rolling_file"
level = "info"
rotation = "daily"
directory = "/robustmq/logs"
prefix = "server"
suffix = "log"
max_log_files = 50

[raft]
kind = "rolling_file"
targets = [{ path = "openraft", level = "info" }]
rotation = "daily"
directory = "/robustmq/logs"
prefix = "raft"
suffix = "log"
max_log_files = 50

[journal]
kind = "rolling_file"
targets = [{ path = "journal_server", level = "info" }]
rotation = "daily"
directory = "/robustmq/logs"
prefix = "journal"
suffix = "log"
max_log_files = 50

[place]
kind = "rolling_file"
targets = [{ path = "meta_service", level = "info" }]
rotation = "daily"
directory = "/robustmq/logs"
prefix = "place"
suffix = "log"
max_log_files = 50
`
}

// SetupWithManager sets up the controller with the Manager.
func (r *RobustMQReconciler) SetupWithManager(mgr ctrl.Manager) error {
	return ctrl.NewControllerManagedBy(mgr).
		For(&robustmqv1alpha1.RobustMQ{}).
		Owns(&appsv1.Deployment{}).
		Owns(&appsv1.StatefulSet{}).
		Owns(&corev1.Service{}).
		Owns(&corev1.ConfigMap{}).
		Complete(r)
}
