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
	"strconv"

	appsv1 "k8s.io/api/apps/v1"
	corev1 "k8s.io/api/core/v1"
	"k8s.io/apimachinery/pkg/api/errors"
	"k8s.io/apimachinery/pkg/api/resource"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/types"
	"sigs.k8s.io/controller-runtime/pkg/controller/controllerutil"
	"sigs.k8s.io/controller-runtime/pkg/log"
	"k8s.io/apimachinery/pkg/util/intstr"
	logr "github.com/go-logr/logr"

	robustmqv1alpha1 "github.com/robustmq/robustmq-operator/api/v1alpha1"
)

// reconcileAllInOne creates or updates StatefulSet for all-in-one deployment
func (r *RobustMQReconciler) reconcileAllInOne(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	logger := log.FromContext(ctx)

	// Set default values if not specified
	replicas := int32(1)
	if robustmq.Spec.AllInOne != nil && robustmq.Spec.AllInOne.Replicas > 0 {
		replicas = robustmq.Spec.AllInOne.Replicas
	}

	statefulSet := &appsv1.StatefulSet{
		ObjectMeta: metav1.ObjectMeta{
			Name:      robustmq.Name,
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "all-in-one",
				"instance":  robustmq.Name,
			},
		},
		Spec: appsv1.StatefulSetSpec{
			Replicas:    &replicas,
			ServiceName: robustmq.Name + "-meta-service",
			Selector: &metav1.LabelSelector{
				MatchLabels: map[string]string{
					"app":      "robustmq",
					"instance": robustmq.Name,
				},
			},
			Template: corev1.PodTemplateSpec{
				ObjectMeta: metav1.ObjectMeta{
					Labels: map[string]string{
						"app":       "robustmq",
						"component": "all-in-one",
						"instance":  robustmq.Name,
					},
				},
				Spec: r.buildAllInOnePodSpec(robustmq),
			},
			VolumeClaimTemplates: r.buildVolumeClaimTemplates(robustmq),
		},
	}

	// Set RobustMQ instance as the owner and controller
	if err := controllerutil.SetControllerReference(robustmq, statefulSet, r.Scheme); err != nil {
		return err
	}

	// Check if StatefulSet already exists
	found := &appsv1.StatefulSet{}
	err := r.Get(ctx, types.NamespacedName{Name: statefulSet.Name, Namespace: statefulSet.Namespace}, found)
	if err != nil && errors.IsNotFound(err) {
		logger.Info("Creating a new StatefulSet", "StatefulSet.Namespace", statefulSet.Namespace, "StatefulSet.Name", statefulSet.Name)
		if err := r.Create(ctx, statefulSet); err != nil {
			return err
		}
	} else if err != nil {
		return err
	} else {
		// Update existing StatefulSet
		found.Spec.Replicas = statefulSet.Spec.Replicas
		found.Spec.Template = statefulSet.Spec.Template
		logger.Info("Updating StatefulSet", "StatefulSet.Namespace", found.Namespace, "StatefulSet.Name", found.Name)
		if err := r.Update(ctx, found); err != nil {
			return err
		}
	}

	return nil
}

// reconcileMicroservices creates or updates deployments for microservices deployment
func (r *RobustMQReconciler) reconcileMicroservices(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	// Deploy Meta Service
	if err := r.reconcileMetaServiceDeployment(ctx, robustmq); err != nil {
		return err
	}

	// Deploy MQTT Broker
	if err := r.reconcileMQTTBrokerDeployment(ctx, robustmq); err != nil {
		return err
	}

	// Deploy Journal Service
	if err := r.reconcileJournalServiceDeployment(ctx, robustmq); err != nil {
		return err
	}

	return nil
}

// reconcileMetaServiceDeployment creates or updates the meta service deployment
func (r *RobustMQReconciler) reconcileMetaServiceDeployment(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	logger := log.FromContext(ctx)

	replicas := int32(3) // Default to 3 replicas for HA
	if robustmq.Spec.Microservices != nil && robustmq.Spec.Microservices.MetaService.Replicas > 0 {
		replicas = robustmq.Spec.Microservices.MetaService.Replicas
	}

	statefulSet := &appsv1.StatefulSet{
		ObjectMeta: metav1.ObjectMeta{
			Name:      robustmq.Name + "-meta",
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "meta",
				"instance":  robustmq.Name,
			},
		},
		Spec: appsv1.StatefulSetSpec{
			Replicas:    &replicas,
			ServiceName: robustmq.Name + "-meta-service",
			Selector: &metav1.LabelSelector{
				MatchLabels: map[string]string{
					"app":       "robustmq",
					"component": "meta",
					"instance":  robustmq.Name,
				},
			},
			Template: corev1.PodTemplateSpec{
				ObjectMeta: metav1.ObjectMeta{
					Labels: map[string]string{
						"app":       "robustmq",
						"component": "meta",
						"instance":  robustmq.Name,
					},
				},
				Spec: r.buildMetaServicePodSpec(robustmq),
			},
			VolumeClaimTemplates: r.buildVolumeClaimTemplates(robustmq),
		},
	}

	return r.createOrUpdateStatefulSet(ctx, robustmq, statefulSet, logger)
}

// reconcileMQTTBrokerDeployment creates or updates the MQTT broker deployment
func (r *RobustMQReconciler) reconcileMQTTBrokerDeployment(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	logger := log.FromContext(ctx)

	replicas := int32(2) // Default to 2 replicas
	if robustmq.Spec.Microservices != nil && robustmq.Spec.Microservices.BrokerService.Replicas > 0 {
		replicas = robustmq.Spec.Microservices.BrokerService.Replicas
	}

	deployment := &appsv1.Deployment{
		ObjectMeta: metav1.ObjectMeta{
			Name:      robustmq.Name + "-mqtt-broker",
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "mqtt-broker",
				"instance":  robustmq.Name,
			},
		},
		Spec: appsv1.DeploymentSpec{
			Replicas: &replicas,
			Selector: &metav1.LabelSelector{
				MatchLabels: map[string]string{
					"app":       "robustmq",
					"component": "mqtt-broker",
					"instance":  robustmq.Name,
				},
			},
			Template: corev1.PodTemplateSpec{
				ObjectMeta: metav1.ObjectMeta{
					Labels: map[string]string{
						"app":       "robustmq",
						"component": "mqtt-broker",
						"instance":  robustmq.Name,
					},
				},
				Spec: r.buildMQTTBrokerPodSpec(robustmq),
			},
		},
	}

	return r.createOrUpdateDeployment(ctx, robustmq, deployment, logger)
}

// reconcileJournalServiceDeployment creates or updates the journal service deployment
func (r *RobustMQReconciler) reconcileJournalServiceDeployment(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ) error {
	logger := log.FromContext(ctx)

	replicas := int32(3) // Default to 3 replicas for data safety
	if robustmq.Spec.Microservices != nil && robustmq.Spec.Microservices.JournalService.Replicas > 0 {
		replicas = robustmq.Spec.Microservices.JournalService.Replicas
	}

	statefulSet := &appsv1.StatefulSet{
		ObjectMeta: metav1.ObjectMeta{
			Name:      robustmq.Name + "-journal",
			Namespace: robustmq.Namespace,
			Labels: map[string]string{
				"app":       "robustmq",
				"component": "journal",
				"instance":  robustmq.Name,
			},
		},
		Spec: appsv1.StatefulSetSpec{
			Replicas:    &replicas,
			ServiceName: robustmq.Name + "-journal-service",
			Selector: &metav1.LabelSelector{
				MatchLabels: map[string]string{
					"app":       "robustmq",
					"component": "journal",
					"instance":  robustmq.Name,
				},
			},
			Template: corev1.PodTemplateSpec{
				ObjectMeta: metav1.ObjectMeta{
					Labels: map[string]string{
						"app":       "robustmq",
						"component": "journal",
						"instance":  robustmq.Name,
					},
				},
				Spec: r.buildJournalServicePodSpec(robustmq),
			},
			VolumeClaimTemplates: r.buildVolumeClaimTemplates(robustmq),
		},
	}

	return r.createOrUpdateStatefulSet(ctx, robustmq, statefulSet, logger)
}

// buildAllInOnePodSpec builds the pod specification for all-in-one deployment
func (r *RobustMQReconciler) buildAllInOnePodSpec(robustmq *robustmqv1alpha1.RobustMQ) corev1.PodSpec {
	image := fmt.Sprintf("%s:%s", robustmq.Spec.Image.Repository, robustmq.Spec.Image.Tag)
	
	podSpec := corev1.PodSpec{
		Containers: []corev1.Container{
			{
				Name:  "robustmq",
				Image: image,
				ImagePullPolicy: robustmq.Spec.Image.PullPolicy,
				Args: []string{"--conf=/robustmq/config/server.toml"},
				Ports: r.buildContainerPorts(robustmq, true),
				VolumeMounts: []corev1.VolumeMount{
					{
						Name:      "config",
						MountPath: "/robustmq/config",
						ReadOnly:  true,
					},
					{
						Name:      "data",
						MountPath: "/robustmq/data",
					},
					{
						Name:      "logs",
						MountPath: "/robustmq/logs",
					},
				},
				Env: []corev1.EnvVar{
					{
						Name:  "ROBUSTMQ_ROLES",
						Value: "meta,broker,journal",
					},
					{
						Name:  "RUST_LOG",
						Value: "info",
					},
				},
				LivenessProbe: &corev1.Probe{
					ProbeHandler: corev1.ProbeHandler{
						HTTPGet: &corev1.HTTPGetAction{
							Path: "/health",
							Port: intstr.FromInt(int(robustmq.Spec.Network.MQTT.WebSocketPort)),
						},
					},
					InitialDelaySeconds: 30,
					PeriodSeconds:       30,
				},
				ReadinessProbe: &corev1.Probe{
					ProbeHandler: corev1.ProbeHandler{
						HTTPGet: &corev1.HTTPGetAction{
							Path: "/health",
							Port: intstr.FromInt(int(robustmq.Spec.Network.MQTT.WebSocketPort)),
						},
					},
					InitialDelaySeconds: 10,
					PeriodSeconds:       10,
				},
			},
		},
		Volumes: []corev1.Volume{
			{
				Name: "config",
				VolumeSource: corev1.VolumeSource{
					ConfigMap: &corev1.ConfigMapVolumeSource{
						LocalObjectReference: corev1.LocalObjectReference{
							Name: robustmq.Name + "-config",
						},
					},
				},
			},
		},
		ImagePullSecrets: robustmq.Spec.Image.PullSecrets,
	}

	// Add resources if specified
	if robustmq.Spec.AllInOne != nil {
		podSpec.Containers[0].Resources = robustmq.Spec.AllInOne.Resources
		podSpec.NodeSelector = robustmq.Spec.AllInOne.NodeSelector
		podSpec.Affinity = robustmq.Spec.AllInOne.Affinity
		podSpec.Tolerations = robustmq.Spec.AllInOne.Tolerations
	}

	return podSpec
}

// buildMetaServicePodSpec builds the pod specification for meta service
func (r *RobustMQReconciler) buildMetaServicePodSpec(robustmq *robustmqv1alpha1.RobustMQ) corev1.PodSpec {
	image := fmt.Sprintf("%s:%s", robustmq.Spec.Image.Repository, robustmq.Spec.Image.Tag)
	
	podSpec := corev1.PodSpec{
		Containers: []corev1.Container{
			{
				Name:  "robustmq-meta",
				Image: image,
				ImagePullPolicy: robustmq.Spec.Image.PullPolicy,
				Args: []string{"--conf=/robustmq/config/server.toml"},
				Ports: []corev1.ContainerPort{
					{
						Name:          "grpc",
						ContainerPort: robustmq.Spec.Network.GRPC.Port,
						Protocol:      corev1.ProtocolTCP,
					},
				},
				VolumeMounts: []corev1.VolumeMount{
					{
						Name:      "config",
						MountPath: "/robustmq/config",
						ReadOnly:  true,
					},
					{
						Name:      "data",
						MountPath: "/robustmq/data",
					},
				},
				Env: []corev1.EnvVar{
					{
						Name:  "ROBUSTMQ_ROLES",
						Value: "meta",
					},
					{
						Name: "POD_NAME",
						ValueFrom: &corev1.EnvVarSource{
							FieldRef: &corev1.ObjectFieldSelector{
								FieldPath: "metadata.name",
							},
						},
					},
				},
			},
		},
		Volumes: []corev1.Volume{
			{
				Name: "config",
				VolumeSource: corev1.VolumeSource{
					ConfigMap: &corev1.ConfigMapVolumeSource{
						LocalObjectReference: corev1.LocalObjectReference{
							Name: robustmq.Name + "-config",
						},
					},
				},
			},
		},
		ImagePullSecrets: robustmq.Spec.Image.PullSecrets,
	}

	// Add resources if specified
	if robustmq.Spec.Microservices != nil {
		podSpec.Containers[0].Resources = robustmq.Spec.Microservices.MetaService.Resources
		podSpec.NodeSelector = robustmq.Spec.Microservices.MetaService.NodeSelector
		podSpec.Affinity = robustmq.Spec.Microservices.MetaService.Affinity
		podSpec.Tolerations = robustmq.Spec.Microservices.MetaService.Tolerations
	}

	return podSpec
}

// buildMQTTBrokerPodSpec builds the pod specification for MQTT broker
func (r *RobustMQReconciler) buildMQTTBrokerPodSpec(robustmq *robustmqv1alpha1.RobustMQ) corev1.PodSpec {
	image := fmt.Sprintf("%s:%s", robustmq.Spec.Image.Repository, robustmq.Spec.Image.Tag)
	
	podSpec := corev1.PodSpec{
		Containers: []corev1.Container{
			{
				Name:  "robustmq-mqtt-broker",
				Image: image,
				ImagePullPolicy: robustmq.Spec.Image.PullPolicy,
				Args: []string{"--conf=/robustmq/config/server.toml"},
				Ports: r.buildContainerPorts(robustmq, false),
				VolumeMounts: []corev1.VolumeMount{
					{
						Name:      "config",
						MountPath: "/robustmq/config",
						ReadOnly:  true,
					},
				},
				Env: []corev1.EnvVar{
					{
						Name:  "ROBUSTMQ_ROLES",
						Value: "broker",
					},
				},
			},
		},
		Volumes: []corev1.Volume{
			{
				Name: "config",
				VolumeSource: corev1.VolumeSource{
					ConfigMap: &corev1.ConfigMapVolumeSource{
						LocalObjectReference: corev1.LocalObjectReference{
							Name: robustmq.Name + "-config",
						},
					},
				},
			},
		},
		ImagePullSecrets: robustmq.Spec.Image.PullSecrets,
	}

	// Add resources if specified
	if robustmq.Spec.Microservices != nil {
		podSpec.Containers[0].Resources = robustmq.Spec.Microservices.BrokerService.Resources
		podSpec.NodeSelector = robustmq.Spec.Microservices.BrokerService.NodeSelector
		podSpec.Affinity = robustmq.Spec.Microservices.BrokerService.Affinity
		podSpec.Tolerations = robustmq.Spec.Microservices.BrokerService.Tolerations
	}

	return podSpec
}

// buildJournalServicePodSpec builds the pod specification for journal service
func (r *RobustMQReconciler) buildJournalServicePodSpec(robustmq *robustmqv1alpha1.RobustMQ) corev1.PodSpec {
	image := fmt.Sprintf("%s:%s", robustmq.Spec.Image.Repository, robustmq.Spec.Image.Tag)
	
	podSpec := corev1.PodSpec{
		Containers: []corev1.Container{
			{
				Name:  "robustmq-journal",
				Image: image,
				ImagePullPolicy: robustmq.Spec.Image.PullPolicy,
				Args: []string{"--conf=/robustmq/config/server.toml"},
				Ports: []corev1.ContainerPort{
					{
						Name:          "journal",
						ContainerPort: 1771,
						Protocol:      corev1.ProtocolTCP,
					},
				},
				VolumeMounts: []corev1.VolumeMount{
					{
						Name:      "config",
						MountPath: "/robustmq/config",
						ReadOnly:  true,
					},
					{
						Name:      "data",
						MountPath: "/robustmq/data",
					},
				},
				Env: []corev1.EnvVar{
					{
						Name:  "ROBUSTMQ_ROLES",
						Value: "journal",
					},
				},
			},
		},
		Volumes: []corev1.Volume{
			{
				Name: "config",
				VolumeSource: corev1.VolumeSource{
					ConfigMap: &corev1.ConfigMapVolumeSource{
						LocalObjectReference: corev1.LocalObjectReference{
							Name: robustmq.Name + "-config",
						},
					},
				},
			},
		},
		ImagePullSecrets: robustmq.Spec.Image.PullSecrets,
	}

	// Add resources if specified
	if robustmq.Spec.Microservices != nil {
		podSpec.Containers[0].Resources = robustmq.Spec.Microservices.JournalService.Resources
		podSpec.NodeSelector = robustmq.Spec.Microservices.JournalService.NodeSelector
		podSpec.Affinity = robustmq.Spec.Microservices.JournalService.Affinity
		podSpec.Tolerations = robustmq.Spec.Microservices.JournalService.Tolerations
	}

	return podSpec
}

// buildContainerPorts builds container ports based on configuration
func (r *RobustMQReconciler) buildContainerPorts(robustmq *robustmqv1alpha1.RobustMQ, includeAll bool) []corev1.ContainerPort {
	var ports []corev1.ContainerPort

	// MQTT ports
	ports = append(ports,
		corev1.ContainerPort{
			Name:          "mqtt-tcp",
			ContainerPort: robustmq.Spec.Network.MQTT.TCPPort,
			Protocol:      corev1.ProtocolTCP,
		},
		corev1.ContainerPort{
			Name:          "mqtt-tls",
			ContainerPort: robustmq.Spec.Network.MQTT.TLSPort,
			Protocol:      corev1.ProtocolTCP,
		},
		corev1.ContainerPort{
			Name:          "mqtt-ws",
			ContainerPort: robustmq.Spec.Network.MQTT.WebSocketPort,
			Protocol:      corev1.ProtocolTCP,
		},
		corev1.ContainerPort{
			Name:          "mqtt-wss",
			ContainerPort: robustmq.Spec.Network.MQTT.WebSocketTLSPort,
			Protocol:      corev1.ProtocolTCP,
		},
	)

	if includeAll {
		// Add other protocol ports for all-in-one deployment
		ports = append(ports,
			corev1.ContainerPort{
				Name:          "kafka",
				ContainerPort: robustmq.Spec.Network.Kafka.Port,
				Protocol:      corev1.ProtocolTCP,
			},
			corev1.ContainerPort{
				Name:          "grpc",
				ContainerPort: robustmq.Spec.Network.GRPC.Port,
				Protocol:      corev1.ProtocolTCP,
			},
			corev1.ContainerPort{
				Name:          "amqp",
				ContainerPort: robustmq.Spec.Network.AMQP.Port,
				Protocol:      corev1.ProtocolTCP,
			},
		)

		// Add Prometheus port if monitoring is enabled
		if robustmq.Spec.Monitoring.Enabled {
			ports = append(ports, corev1.ContainerPort{
				Name:          "prometheus",
				ContainerPort: robustmq.Spec.Monitoring.Prometheus.Port,
				Protocol:      corev1.ProtocolTCP,
			})
		}
	}

	return ports
}

// buildVolumeClaimTemplates builds volume claim templates for StatefulSets
func (r *RobustMQReconciler) buildVolumeClaimTemplates(robustmq *robustmqv1alpha1.RobustMQ) []corev1.PersistentVolumeClaim {
	var templates []corev1.PersistentVolumeClaim

	// Data volume
	templates = append(templates, corev1.PersistentVolumeClaim{
		ObjectMeta: metav1.ObjectMeta{
			Name: "data",
		},
		Spec: corev1.PersistentVolumeClaimSpec{
			AccessModes: robustmq.Spec.Storage.AccessModes,
			Resources: corev1.ResourceRequirements{
				Requests: corev1.ResourceList{
					corev1.ResourceStorage: resource.MustParse(robustmq.Spec.Storage.Size),
				},
			},
			StorageClassName: robustmq.Spec.Storage.StorageClass,
		},
	})

	// Logs volume
	templates = append(templates, corev1.PersistentVolumeClaim{
		ObjectMeta: metav1.ObjectMeta{
			Name: "logs",
		},
		Spec: corev1.PersistentVolumeClaimSpec{
			AccessModes: []corev1.PersistentVolumeAccessMode{corev1.ReadWriteOnce},
			Resources: corev1.ResourceRequirements{
				Requests: corev1.ResourceList{
					corev1.ResourceStorage: resource.MustParse("5Gi"), // Default log volume size
				},
			},
			StorageClassName: robustmq.Spec.Storage.StorageClass,
		},
	})

	return templates
}

// Helper functions to create or update resources

func (r *RobustMQReconciler) createOrUpdateStatefulSet(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, statefulSet *appsv1.StatefulSet, logger logr.Logger) error {
	// Set RobustMQ instance as the owner and controller
	if err := controllerutil.SetControllerReference(robustmq, statefulSet, r.Scheme); err != nil {
		return err
	}

	// Check if StatefulSet already exists
	found := &appsv1.StatefulSet{}
	err := r.Get(ctx, types.NamespacedName{Name: statefulSet.Name, Namespace: statefulSet.Namespace}, found)
	if err != nil && errors.IsNotFound(err) {
		logger.Info("Creating a new StatefulSet", "StatefulSet.Namespace", statefulSet.Namespace, "StatefulSet.Name", statefulSet.Name)
		if err := r.Create(ctx, statefulSet); err != nil {
			return err
		}
	} else if err != nil {
		return err
	} else {
		// Update existing StatefulSet
		found.Spec.Replicas = statefulSet.Spec.Replicas
		found.Spec.Template = statefulSet.Spec.Template
		logger.Info("Updating StatefulSet", "StatefulSet.Namespace", found.Namespace, "StatefulSet.Name", found.Name)
		if err := r.Update(ctx, found); err != nil {
			return err
		}
	}

	return nil
}

func (r *RobustMQReconciler) createOrUpdateDeployment(ctx context.Context, robustmq *robustmqv1alpha1.RobustMQ, deployment *appsv1.Deployment, logger logr.Logger) error {
	// Set RobustMQ instance as the owner and controller
	if err := controllerutil.SetControllerReference(robustmq, deployment, r.Scheme); err != nil {
		return err
	}

	// Check if Deployment already exists
	found := &appsv1.Deployment{}
	err := r.Get(ctx, types.NamespacedName{Name: deployment.Name, Namespace: deployment.Namespace}, found)
	if err != nil && errors.IsNotFound(err) {
		logger.Info("Creating a new Deployment", "Deployment.Namespace", deployment.Namespace, "Deployment.Name", deployment.Name)
		if err := r.Create(ctx, deployment); err != nil {
			return err
		}
	} else if err != nil {
		return err
	} else {
		// Update existing Deployment
		found.Spec.Replicas = deployment.Spec.Replicas
		found.Spec.Template = deployment.Spec.Template
		logger.Info("Updating Deployment", "Deployment.Namespace", found.Namespace, "Deployment.Name", found.Name)
		if err := r.Update(ctx, found); err != nil {
			return err
		}
	}

	return nil
}