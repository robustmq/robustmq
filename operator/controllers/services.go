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

	corev1 "k8s.io/api/core/v1"
	"k8s.io/apimachinery/pkg/api/errors"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/types"
	"k8s.io/apimachinery/pkg/util/intstr"
	"sigs.k8s.io/controller-runtime/pkg/client"
	"sigs.k8s.io/controller-runtime/pkg/controller/controllerutil"
	"sigs.k8s.io/controller-runtime/pkg/log"
	logr "github.com/go-logr/logr"

	robustmqv1alpha1 "github.com/robustmq/robustmq-operator/api/v1alpha1"
)

// reconcileServices creates or updates all necessary services
func (r *RobustMQReconciler) reconcileServices(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	switch robustmq.Spec.DeploymentMode {
	case robustmqv1alpha1.AllInOneMode:
		return r.reconcileAllInOneServices(ctx, robustmq)
	case robustmqv1alpha1.MicroservicesMode:
		return r.reconcileMicroservicesServices(ctx, robustmq)
	default:
		return r.reconcileAllInOneServices(ctx, robustmq)
	}
}

// reconcileAllInOneServices creates services for all-in-one deployment
func (r *RobustMQReconciler) reconcileAllInOneServices(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	// MQTT Service
	if err := r.reconcileMQTTService(ctx, robustmq, robustmq.Name+"-mqtt"); err != nil {
		return err
	}

	// Kafka Service
	if err := r.reconcileKafkaService(ctx, robustmq, robustmq.Name+"-kafka"); err != nil {
		return err
	}

	// gRPC Service
	if err := r.reconcileGRPCService(ctx, robustmq, robustmq.Name+"-grpc"); err != nil {
		return err
	}

	// AMQP Service
	if err := r.reconcileAMQPService(ctx, robustmq, robustmq.Name+"-amqp"); err != nil {
		return err
	}

	// Meta Service (for internal communication)
	if err := r.reconcileMetaService(ctx, robustmq, robustmq.Name+"-meta-service"); err != nil {
		return err
	}

	// Prometheus Service (if monitoring is enabled)
	if robustmq.Spec.Monitoring.Enabled {
		if err := r.reconcilePrometheusService(ctx, robustmq, robustmq.Name+"-prometheus"); err != nil {
			return err
		}
	}

	return nil
}

// reconcileMicroservicesServices creates services for microservices deployment
func (r *RobustMQReconciler) reconcileMicroservicesServices(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	// Meta Service
	if err := r.reconcileMetaService(ctx, robustmq, robustmq.Name+"-meta-service"); err != nil {
		return err
	}

	// MQTT Broker Service
	if err := r.reconcileMQTTService(ctx, robustmq, robustmq.Name+"-mqtt-broker"); err != nil {
		return err
	}

	// Kafka Service
	if err := r.reconcileKafkaService(ctx, robustmq, robustmq.Name+"-kafka"); err != nil {
		return err
	}

	// AMQP Service
	if err := r.reconcileAMQPService(ctx, robustmq, robustmq.Name+"-amqp"); err != nil {
		return err
	}

	// Journal Service
	if err := r.reconcileJournalService(ctx, robustmq, robustmq.Name+"-journal-service"); err != nil {
		return err
	}

	// Prometheus Service (if monitoring is enabled)
	if robustmq.Spec.Monitoring.Enabled {
		if err := r.reconcilePrometheusService(ctx, robustmq, robustmq.Name+"-prometheus"); err != nil {
			return err
		}
	}

	return nil
}

// reconcileMQTTService creates or updates the MQTT service
func (r *RobustMQReconciler) reconcileMQTTService(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, serviceName string) error {
	logger := log.FromContext(ctx)

	service := &corev1.Service{
		ObjectMeta: metav1.ObjectMeta{
			Name:      serviceName,
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "mqtt",
				"instance":  robustmq.Name,
			},
		},
		Spec: corev1.ServiceSpec{
			Type: robustmq.Spec.Network.MQTT.ServiceType,
			Selector: map[string]string{
				"app":      "robustmq",
				"instance": robustmq.Name,
			},
			Ports: []corev1.ServicePort{
				{
					Name:       "mqtt-tcp",
					Port:       robustmq.Spec.Network.MQTT.TCPPort,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Network.MQTT.TCPPort)),
					Protocol:   corev1.ProtocolTCP,
				},
				{
					Name:       "mqtt-tls",
					Port:       robustmq.Spec.Network.MQTT.TLSPort,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Network.MQTT.TLSPort)),
					Protocol:   corev1.ProtocolTCP,
				},
				{
					Name:       "mqtt-ws",
					Port:       robustmq.Spec.Network.MQTT.WebSocketPort,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Network.MQTT.WebSocketPort)),
					Protocol:   corev1.ProtocolTCP,
				},
				{
					Name:       "mqtt-wss",
					Port:       robustmq.Spec.Network.MQTT.WebSocketTLSPort,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Network.MQTT.WebSocketTLSPort)),
					Protocol:   corev1.ProtocolTCP,
				},
			},
		},
	}

	return r.createOrUpdateService(ctx, robustmq, service, logger)
}

// reconcileKafkaService creates or updates the Kafka service
func (r *RobustMQReconciler) reconcileKafkaService(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, serviceName string) error {
	logger := log.FromContext(ctx)

	service := &corev1.Service{
		ObjectMeta: metav1.ObjectMeta{
			Name:      serviceName,
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "kafka",
				"instance":  robustmq.Name,
			},
		},
		Spec: corev1.ServiceSpec{
			Type: robustmq.Spec.Network.Kafka.ServiceType,
			Selector: map[string]string{
				"app":      "robustmq",
				"instance": robustmq.Name,
			},
			Ports: []corev1.ServicePort{
				{
					Name:       "kafka",
					Port:       robustmq.Spec.Network.Kafka.Port,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Network.Kafka.Port)),
					Protocol:   corev1.ProtocolTCP,
				},
			},
		},
	}

	return r.createOrUpdateService(ctx, robustmq, service, logger)
}

// reconcileGRPCService creates or updates the gRPC service
func (r *RobustMQReconciler) reconcileGRPCService(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, serviceName string) error {
	logger := log.FromContext(ctx)

	service := &corev1.Service{
		ObjectMeta: metav1.ObjectMeta{
			Name:      serviceName,
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "grpc",
				"instance":  robustmq.Name,
			},
		},
		Spec: corev1.ServiceSpec{
			Type: robustmq.Spec.Network.GRPC.ServiceType,
			Selector: map[string]string{
				"app":      "robustmq",
				"instance": robustmq.Name,
			},
			Ports: []corev1.ServicePort{
				{
					Name:       "grpc",
					Port:       robustmq.Spec.Network.GRPC.Port,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Network.GRPC.Port)),
					Protocol:   corev1.ProtocolTCP,
				},
			},
		},
	}

	return r.createOrUpdateService(ctx, robustmq, service, logger)
}

// reconcileAMQPService creates or updates the AMQP service
func (r *RobustMQReconciler) reconcileAMQPService(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, serviceName string) error {
	logger := log.FromContext(ctx)

	service := &corev1.Service{
		ObjectMeta: metav1.ObjectMeta{
			Name:      serviceName,
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "amqp",
				"instance":  robustmq.Name,
			},
		},
		Spec: corev1.ServiceSpec{
			Type: robustmq.Spec.Network.AMQP.ServiceType,
			Selector: map[string]string{
				"app":      "robustmq",
				"instance": robustmq.Name,
			},
			Ports: []corev1.ServicePort{
				{
					Name:       "amqp",
					Port:       robustmq.Spec.Network.AMQP.Port,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Network.AMQP.Port)),
					Protocol:   corev1.ProtocolTCP,
				},
			},
		},
	}

	return r.createOrUpdateService(ctx, robustmq, service, logger)
}

// reconcileMetaService creates or updates the meta service
func (r *RobustMQReconciler) reconcileMetaService(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, serviceName string) error {
	logger := log.FromContext(ctx)

	service := &corev1.Service{
		ObjectMeta: metav1.ObjectMeta{
			Name:      serviceName,
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "meta",
				"instance":  robustmq.Name,
			},
		},
		Spec: corev1.ServiceSpec{
			Type:      corev1.ServiceTypeClusterIP,
			ClusterIP: "None", // Headless service for StatefulSet
			Selector: map[string]string{
				"app":      "robustmq",
				"instance": robustmq.Name,
			},
			Ports: []corev1.ServicePort{
				{
					Name:       "grpc",
					Port:       robustmq.Spec.Network.GRPC.Port,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Network.GRPC.Port)),
					Protocol:   corev1.ProtocolTCP,
				},
			},
		},
	}

	return r.createOrUpdateService(ctx, robustmq, service, logger)
}

// reconcileJournalService creates or updates the journal service
func (r *RobustMQReconciler) reconcileJournalService(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, serviceName string) error {
	logger := log.FromContext(ctx)

	service := &corev1.Service{
		ObjectMeta: metav1.ObjectMeta{
			Name:      serviceName,
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "journal",
				"instance":  robustmq.Name,
			},
		},
		Spec: corev1.ServiceSpec{
			Type: corev1.ServiceTypeClusterIP,
			Selector: map[string]string{
				"app":      "robustmq",
				"instance": robustmq.Name,
			},
			Ports: []corev1.ServicePort{
				{
					Name:       "journal",
					Port:       1771,
					TargetPort: intstr.FromInt(1771),
					Protocol:   corev1.ProtocolTCP,
				},
			},
		},
	}

	return r.createOrUpdateService(ctx, robustmq, service, logger)
}

// reconcilePrometheusService creates or updates the Prometheus service
func (r *RobustMQReconciler) reconcilePrometheusService(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, serviceName string) error {
	logger := log.FromContext(ctx)

	service := &corev1.Service{
		ObjectMeta: metav1.ObjectMeta{
			Name:      serviceName,
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "prometheus",
				"instance":  robustmq.Name,
			},
		},
		Spec: corev1.ServiceSpec{
			Type: corev1.ServiceTypeClusterIP,
			Selector: map[string]string{
				"app":      "robustmq",
				"instance": robustmq.Name,
			},
			Ports: []corev1.ServicePort{
				{
					Name:       "prometheus",
					Port:       robustmq.Spec.Monitoring.Prometheus.Port,
					TargetPort: intstr.FromInt(int(robustmq.Spec.Monitoring.Prometheus.Port)),
					Protocol:   corev1.ProtocolTCP,
				},
			},
		},
	}

	return r.createOrUpdateService(ctx, robustmq, service, logger)
}

// createOrUpdateService is a helper function to create or update a service
func (r *RobustMQReconciler) createOrUpdateService(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, service *corev1.Service, logger logr.Logger) error {
	// Set RobustMQ instance as the owner and controller
	if err := controllerutil.SetControllerReference(robustmq, service, r.Scheme); err != nil {
		return err
	}

	// Check if service already exists
	found := &corev1.Service{}
	err := r.Get(ctx, types.NamespacedName{Name: service.Name, Namespace: service.Namespace}, found)
	if err != nil && errors.IsNotFound(err) {
		logger.Info("Creating a new Service", "Service.Namespace", service.Namespace, "Service.Name", service.Name)
		if err := r.Create(ctx, service); err != nil {
			return err
		}
	} else if err != nil {
		return err
	} else {
		// Update existing service
		found.Spec.Ports = service.Spec.Ports
		found.Spec.Type = service.Spec.Type
		logger.Info("Updating Service", "Service.Namespace", found.Namespace, "Service.Name", found.Name)
		if err := r.Update(ctx, found); err != nil {
			return err
		}
	}

	return nil
}