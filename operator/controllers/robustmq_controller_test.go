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
	"time"

	. "github.com/onsi/ginkgo/v2"
	. "github.com/onsi/gomega"
	appsv1 "k8s.io/api/apps/v1"
	corev1 "k8s.io/api/core/v1"
	"k8s.io/apimachinery/pkg/api/errors"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/types"
	"sigs.k8s.io/controller-runtime/pkg/reconcile"

	robustmqv1alpha1 "github.com/robustmq/robustmq-operator/api/v1alpha1"
)

var _ = Describe("RobustMQ Controller", func() {
	Context("When reconciling a resource", func() {
		const resourceName = "test-resource"

		ctx := context.Background()

		typeNamespacedName := types.NamespacedName{
			Name:      resourceName,
			Namespace: "default",
		}
		robustmq := &robustmqv1alpha1.RobustMQ{}

		BeforeEach(func() {
			By("creating the custom resource for the Kind RobustMQ")
			err := k8sClient.Get(ctx, typeNamespacedName, robustmq)
			if err != nil && errors.IsNotFound(err) {
				resource := &robustmqv1alpha1.RobustMQ{
					ObjectMeta: metav1.ObjectMeta{
						Name:      resourceName,
						Namespace: "default",
					},
					Spec: robustmqv1alpha1.RobustMQSpec{
						DeploymentMode: robustmqv1alpha1.AllInOneMode,
						Image: robustmqv1alpha1.ImageSpec{
							Repository: "robustmq/robustmq",
							Tag:        "latest",
							PullPolicy: corev1.PullIfNotPresent,
						},
						AllInOne: &robustmqv1alpha1.AllInOneSpec{
							Replicas: 1,
						},
						Storage: robustmqv1alpha1.StorageSpec{
							Size: "10Gi",
							AccessModes: []corev1.PersistentVolumeAccessMode{
								corev1.ReadWriteOnce,
							},
						},
						Network: robustmqv1alpha1.NetworkSpec{
							MQTT: robustmqv1alpha1.MQTTNetworkSpec{
								TCPPort:           1883,
								TLSPort:           1885,
								WebSocketPort:     8083,
								WebSocketTLSPort:  8084,
								ServiceType:       corev1.ServiceTypeClusterIP,
							},
							Kafka: robustmqv1alpha1.KafkaNetworkSpec{
								Port:        9092,
								ServiceType: corev1.ServiceTypeClusterIP,
							},
							GRPC: robustmqv1alpha1.GRPCNetworkSpec{
								Port:        1228,
								ServiceType: corev1.ServiceTypeClusterIP,
							},
							AMQP: robustmqv1alpha1.AMQPNetworkSpec{
								Port:        5672,
								ServiceType: corev1.ServiceTypeClusterIP,
							},
						},
						Monitoring: robustmqv1alpha1.MonitoringSpec{
							Enabled: true,
							Prometheus: robustmqv1alpha1.PrometheusSpec{
								Enable: true,
								Port:   9091,
								Model:  "pull",
							},
						},
						Security: robustmqv1alpha1.SecuritySpec{
							Auth: robustmqv1alpha1.AuthSpec{
								DefaultUser: "admin",
								StorageType: "placement",
							},
						},
					},
				}
				Expect(k8sClient.Create(ctx, resource)).To(Succeed())
			}
		})

		AfterEach(func() {
			// TODO(user): Cleanup logic after each test, like removing the resource instance.
			resource := &robustmqv1alpha1.RobustMQ{}
			err := k8sClient.Get(ctx, typeNamespacedName, resource)
			Expect(err).NotTo(HaveOccurred())

			By("Cleanup the specific resource instance RobustMQ")
			Expect(k8sClient.Delete(ctx, resource)).To(Succeed())
		})

		It("should successfully reconcile the resource", func() {
			By("Reconciling the created resource")
			controllerReconciler := &RobustMQReconciler{
				Client: k8sClient,
				Scheme: k8sClient.Scheme(),
			}

			_, err := controllerReconciler.Reconcile(ctx, reconcile.Request{
				NamespacedName: typeNamespacedName,
			})
			Expect(err).NotTo(HaveOccurred())

			// TODO(user): Add more specific assertions depending on your controller's reconciliation logic.
			// Example: If you expect a certain status condition to be set, verify it here.
		})

		It("should create StatefulSet for AllInOne mode", func() {
			By("Checking if StatefulSet was created")
			statefulSet := &appsv1.StatefulSet{}
			Eventually(func() error {
				return k8sClient.Get(ctx, typeNamespacedName, statefulSet)
			}, time.Minute, time.Second).Should(Succeed())

			By("Verifying StatefulSet configuration")
			Expect(statefulSet.Spec.Replicas).To(Equal(int32Ptr(1)))
			Expect(statefulSet.Spec.ServiceName).To(Equal(resourceName + "-meta-service"))
		})

		It("should create ConfigMap", func() {
			By("Checking if ConfigMap was created")
			configMap := &corev1.ConfigMap{}
			Eventually(func() error {
				return k8sClient.Get(ctx, types.NamespacedName{
					Name:      resourceName + "-config",
					Namespace: "default",
				}, configMap)
			}, time.Minute, time.Second).Should(Succeed())

			By("Verifying ConfigMap contains server configuration")
			Expect(configMap.Data).To(HaveKey("server.toml"))
			Expect(configMap.Data).To(HaveKey("server-tracing.toml"))
		})

		It("should create Services", func() {
			By("Checking if MQTT service was created")
			mqttService := &corev1.Service{}
			Eventually(func() error {
				return k8sClient.Get(ctx, types.NamespacedName{
					Name:      resourceName + "-mqtt",
					Namespace: "default",
				}, mqttService)
			}, time.Minute, time.Second).Should(Succeed())

			By("Verifying MQTT service configuration")
			Expect(mqttService.Spec.Type).To(Equal(corev1.ServiceTypeClusterIP))
			Expect(len(mqttService.Spec.Ports)).To(BeNumerically(">=", 4)) // TCP, TLS, WebSocket, WebSocket TLS
		})

		It("should update status", func() {
			By("Checking if status is updated")
			Eventually(func() error {
				err := k8sClient.Get(ctx, typeNamespacedName, robustmq)
				if err != nil {
					return err
				}
				if robustmq.Status.Phase == "" {
					return errors.NewNotFound(robustmqv1alpha1.Resource("robustmq"), "status not updated")
				}
				return nil
			}, time.Minute, time.Second).Should(Succeed())

			By("Verifying status phase")
			Expect(robustmq.Status.Phase).To(BeElementOf([]string{
				PhaseInitializing,
				PhaseRunning,
				PhaseUpdating,
			}))
		})
	})
})

func int32Ptr(i int32) *int32 { return &i }