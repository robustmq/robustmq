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
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.Collection;
import java.util.List;
import java.util.UUID;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.common.acl.AccessControlEntry;
import org.apache.kafka.common.acl.AccessControlEntryFilter;
import org.apache.kafka.common.acl.AclBinding;
import org.apache.kafka.common.acl.AclBindingFilter;
import org.apache.kafka.common.acl.AclOperation;
import org.apache.kafka.common.acl.AclPermissionType;
import org.apache.kafka.common.resource.PatternType;
import org.apache.kafka.common.resource.ResourcePattern;
import org.apache.kafka.common.resource.ResourcePatternFilter;
import org.apache.kafka.common.resource.ResourceType;
import org.junit.jupiter.api.Test;

/**
 * Phase 4b: ACLs — DescribeAcls(29), CreateAcls(30), DeleteAcls(31).
 */
class AclTest {

    private static AclBinding topicReadAllow(String topic, String principal) {
        return new AclBinding(
                new ResourcePattern(ResourceType.TOPIC, topic, PatternType.LITERAL),
                new AccessControlEntry(principal, "*", AclOperation.READ, AclPermissionType.ALLOW));
    }

    private static AclBindingFilter filterFor(String topic) {
        return new AclBindingFilter(
                new ResourcePatternFilter(ResourceType.TOPIC, topic, PatternType.LITERAL),
                AccessControlEntryFilter.ANY);
    }

    @Test
    void createDescribeAndDeleteAcl() throws Exception {
        String topic = "acl-topic-" + UUID.randomUUID();
        String principal = "User:alice-" + UUID.randomUUID();
        AclBinding binding = topicReadAllow(topic, principal);

        try (Admin admin = Support.newAdmin()) {
            admin.createAcls(List.of(binding)).all().get();

            Collection<AclBinding> described = admin.describeAcls(filterFor(topic)).values().get();
            assertEquals(1, described.size(), "the created ACL should be described");
            AclBinding found = described.iterator().next();
            assertEquals(topic, found.pattern().name());
            assertEquals(principal, found.entry().principal());
            assertEquals(AclOperation.READ, found.entry().operation());
            assertEquals(AclPermissionType.ALLOW, found.entry().permissionType());

            admin.deleteAcls(List.of(filterFor(topic))).all().get();

            assertTrue(admin.describeAcls(filterFor(topic)).values().get().isEmpty(),
                    "the deleted ACL should no longer be described");
        }
    }

    @Test
    void describeWithNoMatchIsEmpty() throws Exception {
        try (Admin admin = Support.newAdmin()) {
            Collection<AclBinding> described =
                    admin.describeAcls(filterFor("acl-absent-" + UUID.randomUUID())).values().get();
            assertTrue(described.isEmpty(), "a filter matching nothing returns no ACLs");
        }
    }
}
