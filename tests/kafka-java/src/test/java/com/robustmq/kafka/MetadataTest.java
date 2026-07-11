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

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Collection;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.DescribeClusterResult;
import org.apache.kafka.common.Node;
import org.junit.jupiter.api.Test;

class MetadataTest {

    @Test
    void describeClusterReturnsBrokersAndController() throws Exception {
        try (Admin admin = Support.newAdmin()) {
            DescribeClusterResult cluster = admin.describeCluster();

            Collection<Node> nodes = cluster.nodes().get();
            assertFalse(nodes.isEmpty(), "cluster returned no brokers");
            for (Node node : nodes) {
                assertFalse(node.host().isEmpty(), "broker host is empty");
                assertTrue(node.port() > 0, "broker port is not set");
            }

            Node controller = cluster.controller().get();
            assertNotNull(controller, "no controller reported");
            assertTrue(
                    nodes.stream().anyMatch(n -> n.id() == controller.id()),
                    "controller " + controller.id() + " is not in the broker list");
        }
    }
}
