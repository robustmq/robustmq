/*
 * Copyright 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package com.robustmq.kafka;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.CreateTopicsOptions;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.admin.TopicDescription;
import org.apache.kafka.common.errors.InvalidReplicaAssignmentException;
import org.apache.kafka.common.errors.TopicExistsException;
import org.junit.jupiter.api.Test;

class CreateTopicsTest {

    private static String name() {
        return "it-create-" + UUID.randomUUID();
    }

    @Test
    void duplicateTopicIsRejected() throws Exception {
        String topic = name();
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();

            ExecutionException ex = assertThrows(
                    ExecutionException.class,
                    () -> admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get());
            assertInstanceOf(TopicExistsException.class, ex.getCause(), "expected TopicExists");
        }
    }

    @Test
    void validateOnlyDoesNotCreate() throws Exception {
        String topic = name();
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(
                    List.of(new NewTopic(topic, 2, (short) 1)),
                    new CreateTopicsOptions().validateOnly(true)).all().get();

            assertFalse(
                    admin.listTopics().names().get().contains(topic),
                    "validateOnly must not actually create the topic");
        }
    }

    @Test
    void createMultipleTopicsAtOnce() throws Exception {
        String a = name();
        String b = name();
        try (Admin admin = Support.newAdmin()) {
            admin.createTopics(List.of(
                    new NewTopic(a, 1, (short) 1),
                    new NewTopic(b, 3, (short) 1))).all().get();

            Map<String, TopicDescription> descs =
                    admin.describeTopics(List.of(a, b)).allTopicNames().get();
            assertEquals(1, descs.get(a).partitions().size());
            assertEquals(3, descs.get(b).partitions().size());
        }
    }

    @Test
    void defaultsUsedWhenPartitionsAndReplicationUnspecified() throws Exception {
        String topic = name();
        try (Admin admin = Support.newAdmin()) {
            // Optional.empty() sends -1/-1, asking the broker for its cluster defaults.
            admin.createTopics(List.of(new NewTopic(topic, Optional.empty(), Optional.empty())))
                    .all().get();

            TopicDescription desc =
                    admin.describeTopics(List.of(topic)).allTopicNames().get().get(topic);
            // Cluster default is 1 partition / 1 replica.
            assertEquals(1, desc.partitions().size(), "unexpected default partition count");
            assertEquals(1, desc.partitions().get(0).replicas().size(), "unexpected default replicas");
        }
    }

    @Test
    void manualReplicaAssignmentIsRejected() throws Exception {
        String topic = name();
        try (Admin admin = Support.newAdmin()) {
            // Explicit replica placement isn't supported; the broker assigns replicas itself.
            NewTopic manual = new NewTopic(topic, Map.of(0, List.of(0)));
            ExecutionException ex = assertThrows(
                    ExecutionException.class,
                    () -> admin.createTopics(List.of(manual)).all().get());
            assertInstanceOf(
                    InvalidReplicaAssignmentException.class,
                    ex.getCause(),
                    "expected InvalidReplicaAssignment, got " + ex.getCause());
        }
    }
}
