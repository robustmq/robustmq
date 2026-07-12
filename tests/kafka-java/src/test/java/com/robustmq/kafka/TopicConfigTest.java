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
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.AlterConfigOp;
import org.apache.kafka.clients.admin.Config;
import org.apache.kafka.clients.admin.ConfigEntry;
import org.apache.kafka.clients.admin.NewPartitions;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.admin.TopicDescription;
import org.apache.kafka.common.config.ConfigResource;
import org.apache.kafka.common.errors.InvalidConfigurationException;
import org.apache.kafka.common.errors.InvalidPartitionsException;
import org.junit.jupiter.api.Test;

/**
 * Phase 2: topic configuration and partition growth — CreatePartitions(37),
 * DescribeConfigs(32), IncrementalAlterConfigs(44) via the AdminClient.
 */
class TopicConfigTest {

    private static String name() {
        return "it-cfg-" + UUID.randomUUID();
    }

    private static void createTopic(Admin admin, String topic, int partitions) throws Exception {
        admin.createTopics(List.of(new NewTopic(topic, partitions, (short) 1))).all().get();
    }

    // ---- CreatePartitions --------------------------------------------------

    @Test
    void increasePartitionsIsReflectedInMetadata() throws Exception {
        String topic = name();
        try (Admin admin = Support.newAdmin()) {
            createTopic(admin, topic, 1);
            admin.createPartitions(Map.of(topic, NewPartitions.increaseTo(3))).all().get();

            TopicDescription desc =
                    admin.describeTopics(List.of(topic)).allTopicNames().get().get(topic);
            assertEquals(3, desc.partitions().size(), "partition count should grow to 3");
        }
    }

    @Test
    void shrinkingPartitionsIsRejected() throws Exception {
        String topic = name();
        try (Admin admin = Support.newAdmin()) {
            createTopic(admin, topic, 3);
            // increaseTo(2) is below the current count of 3 — not allowed.
            ExecutionException ex = assertThrows(ExecutionException.class,
                    () -> admin.createPartitions(Map.of(topic, NewPartitions.increaseTo(2)))
                            .all().get());
            assertInstanceOf(InvalidPartitionsException.class, ex.getCause());
        }
    }

    // ---- DescribeConfigs ---------------------------------------------------

    @Test
    void describeTopicConfigsReturnsValues() throws Exception {
        String topic = name();
        ConfigResource resource = new ConfigResource(ConfigResource.Type.TOPIC, topic);
        try (Admin admin = Support.newAdmin()) {
            createTopic(admin, topic, 1);
            Config config = admin.describeConfigs(List.of(resource)).all().get().get(resource);
            assertNotNull(config, "topic config should be described");
            ConfigEntry retention = config.get("retention.ms");
            assertNotNull(retention, "retention.ms should be present");
            assertNotNull(retention.value(), "retention.ms should have a value");
        }
    }

    // ---- IncrementalAlterConfigs ------------------------------------------

    @Test
    void incrementalSetThenDescribeRoundTrips() throws Exception {
        String topic = name();
        ConfigResource resource = new ConfigResource(ConfigResource.Type.TOPIC, topic);
        try (Admin admin = Support.newAdmin()) {
            createTopic(admin, topic, 1);

            AlterConfigOp op = new AlterConfigOp(
                    new ConfigEntry("retention.ms", "7200000"), AlterConfigOp.OpType.SET);
            admin.incrementalAlterConfigs(Map.of(resource, List.of(op))).all().get();

            Config config = admin.describeConfigs(List.of(resource)).all().get().get(resource);
            assertEquals("7200000", config.get("retention.ms").value(),
                    "the altered value should be read back");
        }
    }

    @Test
    void incrementalDeleteRevertsToDefault() throws Exception {
        String topic = name();
        ConfigResource resource = new ConfigResource(ConfigResource.Type.TOPIC, topic);
        try (Admin admin = Support.newAdmin()) {
            createTopic(admin, topic, 1);

            String original = admin.describeConfigs(List.of(resource)).all().get()
                    .get(resource).get("retention.ms").value();

            admin.incrementalAlterConfigs(Map.of(resource, List.of(new AlterConfigOp(
                    new ConfigEntry("retention.ms", "7200000"), AlterConfigOp.OpType.SET))))
                    .all().get();
            admin.incrementalAlterConfigs(Map.of(resource, List.of(new AlterConfigOp(
                    new ConfigEntry("retention.ms", null), AlterConfigOp.OpType.DELETE))))
                    .all().get();

            String after = admin.describeConfigs(List.of(resource)).all().get()
                    .get(resource).get("retention.ms").value();
            assertEquals(original, after, "deleting a config should revert it to the default");
        }
    }

    @Test
    void unknownConfigNameIsRejected() throws Exception {
        String topic = name();
        ConfigResource resource = new ConfigResource(ConfigResource.Type.TOPIC, topic);
        try (Admin admin = Support.newAdmin()) {
            createTopic(admin, topic, 1);
            AlterConfigOp op = new AlterConfigOp(
                    new ConfigEntry("not.a.real.config", "x"), AlterConfigOp.OpType.SET);
            ExecutionException ex = assertThrows(ExecutionException.class,
                    () -> admin.incrementalAlterConfigs(Map.of(resource, List.of(op))).all().get());
            assertInstanceOf(InvalidConfigurationException.class, ex.getCause(),
                    "an unrecognized config name must be rejected");
        }
    }

    @Test
    void describeConfigsOnUnknownTopicFails() throws Exception {
        ConfigResource resource = new ConfigResource(ConfigResource.Type.TOPIC, name());
        try (Admin admin = Support.newAdmin()) {
            ExecutionException ex = assertThrows(ExecutionException.class,
                    () -> admin.describeConfigs(List.of(resource)).all().get());
            assertTrue(ex.getCause() instanceof org.apache.kafka.common.errors.UnknownTopicOrPartitionException,
                    "describing configs of a missing topic must fail, got " + ex.getCause());
        }
    }
}
