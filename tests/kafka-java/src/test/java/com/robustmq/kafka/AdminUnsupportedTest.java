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

import static org.junit.jupiter.api.Assertions.assertThrows;

import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.AdminClientConfig;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.common.ElectionType;
import org.apache.kafka.common.TopicPartition;
import org.junit.jupiter.api.Test;

/**
 * Phase 7: operations RobustMQ intentionally does not support (replica placement
 * and leader election are internal) must fail cleanly and promptly — the client
 * gets an error rather than hanging.
 */
class AdminUnsupportedTest {

    /** Short timeouts so an intentionally-unsupported op can never stall the suite. */
    private static Admin fastAdmin() {
        return Support.newAdmin(Map.of(
                AdminClientConfig.REQUEST_TIMEOUT_MS_CONFIG, 4000,
                AdminClientConfig.DEFAULT_API_TIMEOUT_MS_CONFIG, 8000));
    }

    @Test
    void listPartitionReassignmentsIsRejected() throws Exception {
        try (Admin admin = fastAdmin()) {
            assertThrows(ExecutionException.class,
                    () -> admin.listPartitionReassignments().reassignments().get(),
                    "replica reassignment is managed internally, not via the Kafka protocol");
        }
    }

    @Test
    void electLeadersIsRejected() throws Exception {
        String topic = "it-elect-" + UUID.randomUUID();
        try (Admin admin = fastAdmin()) {
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
            Set<TopicPartition> partitions = Set.of(new TopicPartition(topic, 0));
            assertThrows(ExecutionException.class,
                    () -> admin.electLeaders(ElectionType.PREFERRED, partitions).partitions().get(),
                    "leader election is automatic; explicit election is unsupported");
        }
    }

    @Test
    void updateFeaturesIsRejected() throws Exception {
        try (Admin admin = fastAdmin()) {
            var update = new org.apache.kafka.clients.admin.FeatureUpdate(
                    (short) 1, org.apache.kafka.clients.admin.FeatureUpdate.UpgradeType.UPGRADE);
            assertThrows(ExecutionException.class,
                    () -> admin.updateFeatures(Map.of("metadata.version", update),
                            new org.apache.kafka.clients.admin.UpdateFeaturesOptions()).all().get(),
                    "there is no KRaft-style feature metadata to update");
        }
    }
}
