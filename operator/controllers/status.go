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

	appsv1 "k8s.io/api/apps/v1"
	corev1 "k8s.io/api/core/v1"
	"k8s.io/apimachinery/pkg/api/errors"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/types"
	"sigs.k8s.io/controller-runtime/pkg/log"

	robustmqv1alpha1 "github.com/robustmq/robustmq-operator/api/v1alpha1"
)

// updateStatus updates the status of the RobustMQ instance
func (r *RobustMQReconciler) updateStatus(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	logger := log.FromContext(ctx)

	// Initialize status fields if empty
	if robustmq.Status.DeploymentStatuses == nil {
		robustmq.Status.DeploymentStatuses = make(map[string]robustmqv1alpha1.DeploymentStatus)
	}
	if robustmq.Status.ServiceStatuses == nil {
		robustmq.Status.ServiceStatuses = make(map[string]robustmqv1alpha1.ServiceStatus)
	}

	// Update deployment statuses based on deployment mode
	switch robustmq.Spec.DeploymentMode {
	case robustmqv1alpha1.AllInOneMode:
		if err := r.updateAllInOneStatus(ctx, robustmq); err != nil {
			logger.Error(err, "Failed to update all-in-one status")
			return err
		}
	case robustmqv1alpha1.MicroservicesMode:
		if err := r.updateMicroservicesStatus(ctx, robustmq); err != nil {
			logger.Error(err, "Failed to update microservices status")
			return err
		}
	}

	// Update service statuses
	if err := r.updateServiceStatuses(ctx, robustmq); err != nil {
		logger.Error(err, "Failed to update service statuses")
		return err
	}

	// Update cluster info
	r.updateClusterInfo(robustmq)

	// Determine overall phase
	r.updateOverallPhase(robustmq)

	// Update conditions
	r.updateConditions(robustmq)

	// Update the status
	if err := r.Status().Update(ctx, robustmq); err != nil {
		logger.Error(err, "Failed to update RobustMQ status")
		return err
	}

	return nil
}

// updateAllInOneStatus updates status for all-in-one deployment
func (r *RobustMQReconciler) updateAllInOneStatus(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	// Get StatefulSet status
	statefulSet := &appsv1.StatefulSet{}
	err := r.Get(ctx, types.NamespacedName{Name: robustmq.Name, Namespace: robustmq.Namespace}, statefulSet)
	if err != nil {
		if errors.IsNotFound(err) {
			// StatefulSet not found, set status accordingly
			robustmq.Status.DeploymentStatuses["all-in-one"] = robustmqv1alpha1.DeploymentStatus{
				Replicas:          0,
				ReadyReplicas:     0,
				AvailableReplicas: 0,
			}
			return nil
		}
		return err
	}

	// Update deployment status
	robustmq.Status.DeploymentStatuses["all-in-one"] = robustmqv1alpha1.DeploymentStatus{
		Replicas:          statefulSet.Status.Replicas,
		ReadyReplicas:     statefulSet.Status.ReadyReplicas,
		AvailableReplicas: statefulSet.Status.ReadyReplicas, // StatefulSet doesn't have AvailableReplicas
	}

	return nil
}

// updateMicroservicesStatus updates status for microservices deployment
func (r *RobustMQReconciler) updateMicroservicesStatus(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	// Update Meta Service status
	if err := r.updateStatefulSetStatus(ctx, robustmq, robustmq.Name+"-meta", "meta-service"); err != nil {
		return err
	}

	// Update MQTT Broker status
	if err := r.updateDeploymentStatus(ctx, robustmq, robustmq.Name+"-mqtt-broker", "mqtt-broker"); err != nil {
		return err
	}

	// Update Journal Service status
	if err := r.updateStatefulSetStatus(ctx, robustmq, robustmq.Name+"-journal", "journal-service"); err != nil {
		return err
	}

	return nil
}

// updateStatefulSetStatus updates status for a specific StatefulSet
func (r *RobustMQReconciler) updateStatefulSetStatus(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, name, component string) error {
	statefulSet := &appsv1.StatefulSet{}
	err := r.Get(ctx, types.NamespacedName{Name: name, Namespace: robustmq.Namespace}, statefulSet)
	if err != nil {
		if errors.IsNotFound(err) {
			robustmq.Status.DeploymentStatuses[component] = robustmqv1alpha1.DeploymentStatus{
				Replicas:          0,
				ReadyReplicas:     0,
				AvailableReplicas: 0,
			}
			return nil
		}
		return err
	}

	robustmq.Status.DeploymentStatuses[component] = robustmqv1alpha1.DeploymentStatus{
		Replicas:          statefulSet.Status.Replicas,
		ReadyReplicas:     statefulSet.Status.ReadyReplicas,
		AvailableReplicas: statefulSet.Status.ReadyReplicas,
	}

	return nil
}

// updateDeploymentStatus updates status for a specific Deployment
func (r *RobustMQReconciler) updateDeploymentStatus(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, name, component string) error {
	deployment := &appsv1.Deployment{}
	err := r.Get(ctx, types.NamespacedName{Name: name, Namespace: robustmq.Namespace}, deployment)
	if err != nil {
		if errors.IsNotFound(err) {
			robustmq.Status.DeploymentStatuses[component] = robustmqv1alpha1.DeploymentStatus{
				Replicas:          0,
				ReadyReplicas:     0,
				AvailableReplicas: 0,
			}
			return nil
		}
		return err
	}

	robustmq.Status.DeploymentStatuses[component] = robustmqv1alpha1.DeploymentStatus{
		Replicas:          deployment.Status.Replicas,
		ReadyReplicas:     deployment.Status.ReadyReplicas,
		AvailableReplicas: deployment.Status.AvailableReplicas,
	}

	return nil
}

// updateServiceStatuses updates the status of all services
func (r *RobustMQReconciler) updateServiceStatuses(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	serviceNames := []string{}

	// Determine service names based on deployment mode
	switch robustmq.Spec.DeploymentMode {
	case robustmqv1alpha1.AllInOneMode:
		serviceNames = []string{
			robustmq.Name + "-mqtt",
			robustmq.Name + "-kafka",
			robustmq.Name + "-grpc",
			robustmq.Name + "-amqp",
			robustmq.Name + "-meta-service",
		}
		if robustmq.Spec.Monitoring.Enabled {
			serviceNames = append(serviceNames, robustmq.Name+"-prometheus")
		}
	case robustmqv1alpha1.MicroservicesMode:
		serviceNames = []string{
			robustmq.Name + "-meta-service",
			robustmq.Name + "-mqtt-broker",
			robustmq.Name + "-kafka",
			robustmq.Name + "-amqp",
			robustmq.Name + "-journal-service",
		}
		if robustmq.Spec.Monitoring.Enabled {
			serviceNames = append(serviceNames, robustmq.Name+"-prometheus")
		}
	}

	// Update status for each service
	for _, serviceName := range serviceNames {
		if err := r.updateSingleServiceStatus(ctx, robustmq, serviceName); err != nil {
			return err
		}
	}

	return nil
}

// updateSingleServiceStatus updates the status of a single service
func (r *RobustMQReconciler) updateSingleServiceStatus(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, serviceName string) error {
	service := &corev1.Service{}
	err := r.Get(ctx, types.NamespacedName{Name: serviceName, Namespace: robustmq.Namespace}, service)
	if err != nil {
		if errors.IsNotFound(err) {
			// Service not found
			robustmq.Status.ServiceStatuses[serviceName] = robustmqv1alpha1.ServiceStatus{}
			return nil
		}
		return err
	}

	// Update service status
	serviceStatus := robustmqv1alpha1.ServiceStatus{
		ClusterIP: service.Spec.ClusterIP,
	}

	// Update external IPs
	if len(service.Spec.ExternalIPs) > 0 {
		serviceStatus.ExternalIPs = service.Spec.ExternalIPs
	}

	// Update load balancer ingress
	if service.Spec.Type == corev1.ServiceTypeLoadBalancer {
		serviceStatus.LoadBalancerIngress = service.Status.LoadBalancer.Ingress
	}

	robustmq.Status.ServiceStatuses[serviceName] = serviceStatus

	return nil
}

// updateClusterInfo updates the cluster information
func (r *RobustMQReconciler) updateClusterInfo(robustmq *robustmqv1alpha1.RobustMQ) {
	// Set cluster name
	clusterName := robustmq.Name
	if robustmq.Spec.Config.Additional != nil {
		if name, exists := robustmq.Spec.Config.Additional["cluster_name"]; exists {
			clusterName = name
		}
	}

	robustmq.Status.ClusterInfo.ClusterName = clusterName

	// Update nodes information based on deployment mode
	var nodes []robustmqv1alpha1.NodeInfo

	switch robustmq.Spec.DeploymentMode {
	case robustmqv1alpha1.AllInOneMode:
		// For all-in-one, each replica is a node with all roles
		if status, exists := robustmq.Status.DeploymentStatuses["all-in-one"]; exists {
			for i := int32(0); i < status.ReadyReplicas; i++ {
				nodes = append(nodes, robustmqv1alpha1.NodeInfo{
					ID:      fmt.Sprintf("%s-%d", robustmq.Name, i),
					Roles:   []robustmqv1alpha1.ServiceRole{robustmqv1alpha1.MetaRole, robustmqv1alpha1.BrokerRole, robustmqv1alpha1.JournalRole},
					Address: fmt.Sprintf("%s-%d.%s-meta-service.%s.svc.cluster.local:%d", robustmq.Name, i, robustmq.Name, robustmq.Namespace, robustmq.Spec.Network.GRPC.Port),
					Status:  "Running",
				})
			}
		}
	case robustmqv1alpha1.MicroservicesMode:
		// Add meta service nodes
		if status, exists := robustmq.Status.DeploymentStatuses["meta-service"]; exists {
			for i := int32(0); i < status.ReadyReplicas; i++ {
				nodes = append(nodes, robustmqv1alpha1.NodeInfo{
					ID:      fmt.Sprintf("%s-meta-%d", robustmq.Name, i),
					Roles:   []robustmqv1alpha1.ServiceRole{robustmqv1alpha1.MetaRole},
					Address: fmt.Sprintf("%s-meta-%d.%s-meta-service.%s.svc.cluster.local:%d", robustmq.Name, i, robustmq.Name, robustmq.Namespace, robustmq.Spec.Network.GRPC.Port),
					Status:  "Running",
				})
			}
		}

		// Add broker service nodes
		if status, exists := robustmq.Status.DeploymentStatuses["mqtt-broker"]; exists {
			for i := int32(0); i < status.ReadyReplicas; i++ {
				nodes = append(nodes, robustmqv1alpha1.NodeInfo{
					ID:      fmt.Sprintf("%s-broker-%d", robustmq.Name, i),
					Roles:   []robustmqv1alpha1.ServiceRole{robustmqv1alpha1.BrokerRole},
					Address: fmt.Sprintf("%s-mqtt-broker-%d", robustmq.Name, i),
					Status:  "Running",
				})
			}
		}

		// Add journal service nodes
		if status, exists := robustmq.Status.DeploymentStatuses["journal-service"]; exists {
			for i := int32(0); i < status.ReadyReplicas; i++ {
				nodes = append(nodes, robustmqv1alpha1.NodeInfo{
					ID:      fmt.Sprintf("%s-journal-%d", robustmq.Name, i),
					Roles:   []robustmqv1alpha1.ServiceRole{robustmqv1alpha1.JournalRole},
					Address: fmt.Sprintf("%s-journal-%d.%s-journal-service.%s.svc.cluster.local:1771", robustmq.Name, i, robustmq.Name, robustmq.Namespace),
					Status:  "Running",
				})
			}
		}
	}

	robustmq.Status.ClusterInfo.Nodes = nodes

	// Set meta leader (for simplicity, assume first meta node is leader)
	if len(nodes) > 0 {
		for _, node := range nodes {
			for _, role := range node.Roles {
				if role == robustmqv1alpha1.MetaRole {
					robustmq.Status.ClusterInfo.MetaLeader = node.ID
					break
				}
			}
			if robustmq.Status.ClusterInfo.MetaLeader != "" {
				break
			}
		}
	}
}

// updateOverallPhase determines the overall phase of the RobustMQ cluster
func (r *RobustMQReconciler) updateOverallPhase(robustmq *robustmqv1alpha1.RobustMQ) {
	totalReplicas := int32(0)
	readyReplicas := int32(0)

	// Count total and ready replicas
	for _, status := range robustmq.Status.DeploymentStatuses {
		totalReplicas += status.Replicas
		readyReplicas += status.ReadyReplicas
	}

	if totalReplicas == 0 {
		robustmq.Status.Phase = PhaseInitializing
	} else if readyReplicas == 0 {
		robustmq.Status.Phase = PhaseFailed
	} else if readyReplicas < totalReplicas {
		robustmq.Status.Phase = PhaseUpdating
	} else {
		robustmq.Status.Phase = PhaseRunning
	}
}

// updateConditions updates the conditions in the status
func (r *RobustMQReconciler) updateConditions(robustmq *robustmqv1alpha1.RobustMQ) {
	now := metav1.Now()

	// Ready condition
	readyCondition := metav1.Condition{
		Type:               ConditionTypeReady,
		LastTransitionTime: now,
		ObservedGeneration: robustmq.Generation,
	}

	if robustmq.Status.Phase == PhaseRunning {
		readyCondition.Status = metav1.ConditionTrue
		readyCondition.Reason = "AllComponentsReady"
		readyCondition.Message = "All RobustMQ components are ready"
	} else {
		readyCondition.Status = metav1.ConditionFalse
		readyCondition.Reason = "ComponentsNotReady"
		readyCondition.Message = fmt.Sprintf("RobustMQ is in %s phase", robustmq.Status.Phase)
	}

	// Update or add the condition
	r.setCondition(robustmq, readyCondition)

	// ConfigMap Ready condition
	configMapCondition := metav1.Condition{
		Type:               ConditionTypeConfigMapReady,
		Status:             metav1.ConditionTrue,
		Reason:             "ConfigMapCreated",
		Message:            "ConfigMap has been created successfully",
		LastTransitionTime: now,
		ObservedGeneration: robustmq.Generation,
	}
	r.setCondition(robustmq, configMapCondition)

	// Service Ready condition
	serviceCondition := metav1.Condition{
		Type:               ConditionTypeServiceReady,
		Status:             metav1.ConditionTrue,
		Reason:             "ServicesCreated",
		Message:            "All services have been created successfully",
		LastTransitionTime: now,
		ObservedGeneration: robustmq.Generation,
	}
	r.setCondition(robustmq, serviceCondition)
}

// setCondition sets or updates a condition in the status
func (r *RobustMQReconciler) setCondition(robustmq *robustmqv1alpha1.RobustMQ, condition metav1.Condition) {
	if robustmq.Status.Conditions == nil {
		robustmq.Status.Conditions = []metav1.Condition{}
	}

	// Find existing condition
	for i, existingCondition := range robustmq.Status.Conditions {
		if existingCondition.Type == condition.Type {
			// Update existing condition
			if existingCondition.Status != condition.Status {
				condition.LastTransitionTime = metav1.Now()
			} else {
				condition.LastTransitionTime = existingCondition.LastTransitionTime
			}
			robustmq.Status.Conditions[i] = condition
			return
		}
	}

	// Add new condition
	robustmq.Status.Conditions = append(robustmq.Status.Conditions, condition)
}