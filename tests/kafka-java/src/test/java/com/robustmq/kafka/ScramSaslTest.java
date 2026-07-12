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
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.clients.admin.NewTopic;
import org.apache.kafka.clients.admin.ScramCredentialInfo;
import org.apache.kafka.clients.admin.ScramMechanism;
import org.apache.kafka.clients.admin.UserScramCredentialDeletion;
import org.apache.kafka.clients.admin.UserScramCredentialUpsertion;
import org.apache.kafka.clients.admin.UserScramCredentialsDescription;
import org.apache.kafka.clients.producer.KafkaProducer;
import org.apache.kafka.clients.producer.ProducerConfig;
import org.apache.kafka.clients.producer.ProducerRecord;
import org.apache.kafka.common.errors.SaslAuthenticationException;
import org.junit.jupiter.api.Test;

/**
 * Phase 3: SASL/SCRAM authentication — AlterUserScramCredentials(51),
 * DescribeUserScramCredentials(50), SaslHandshake(17), SaslAuthenticate(36).
 * Credentials are managed over the plaintext admin connection, then a
 * SASL_PLAINTEXT client authenticates with them.
 */
class ScramSaslTest {

    private static final int ITERATIONS = 8192;

    private static String user() {
        return "user-" + UUID.randomUUID();
    }

    private static String topicName() {
        return "it-sasl-" + UUID.randomUUID();
    }

    /** Client overrides that make a client authenticate over SASL_PLAINTEXT/SCRAM-SHA-256. */
    private static Map<String, Object> scram(String user, String password) {
        String jaas = "org.apache.kafka.common.security.scram.ScramLoginModule required "
                + "username=\"" + user + "\" password=\"" + password + "\";";
        return Map.of(
                "security.protocol", "SASL_PLAINTEXT",
                "sasl.mechanism", "SCRAM-SHA-256",
                "sasl.jaas.config", jaas);
    }

    private static void createUser(Admin admin, String user, String password) throws Exception {
        admin.alterUserScramCredentials(List.of(new UserScramCredentialUpsertion(
                user, new ScramCredentialInfo(ScramMechanism.SCRAM_SHA_256, ITERATIONS), password)))
                .all().get();
    }

    @Test
    void scramCredentialLifecycle() throws Exception {
        String user = user();
        try (Admin admin = Support.newAdmin()) {
            createUser(admin, user, "pw-" + user);

            Map<String, UserScramCredentialsDescription> described =
                    admin.describeUserScramCredentials().all().get();
            assertTrue(described.containsKey(user), "created SCRAM user should be described");
            assertEquals(ScramMechanism.SCRAM_SHA_256,
                    described.get(user).credentialInfos().get(0).mechanism());

            admin.alterUserScramCredentials(List.of(
                    new UserScramCredentialDeletion(user, ScramMechanism.SCRAM_SHA_256)))
                    .all().get();

            assertFalse(admin.describeUserScramCredentials().all().get().containsKey(user),
                    "deleted SCRAM user should no longer be described");
        }
    }

    @Test
    void authenticatedClientCanProduceAndBeReadBack() throws Exception {
        String user = user();
        String password = "pw-" + user;
        String topic = topicName();
        try (Admin admin = Support.newAdmin()) {
            createUser(admin, user, password);
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
        }

        // Produce over an authenticated SASL_PLAINTEXT connection.
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer(scram(user, password))) {
            for (int i = 0; i < 3; i++) {
                producer.send(new ProducerRecord<>(topic, 0, null, ("v" + i).getBytes())).get();
            }
        }
        assertEquals(3, Support.consumeAllFromBeginning(topic, 3).size(),
                "records written by the authenticated client must be readable");
    }

    @Test
    void wrongPasswordIsRejected() throws Exception {
        String user = user();
        String topic = topicName();
        try (Admin admin = Support.newAdmin()) {
            createUser(admin, user, "correct-password");
            admin.createTopics(List.of(new NewTopic(topic, 1, (short) 1))).all().get();
        }

        Map<String, Object> badAuth = new java.util.HashMap<>(scram(user, "wrong-password"));
        badAuth.put(ProducerConfig.MAX_BLOCK_MS_CONFIG, "8000");
        try (KafkaProducer<byte[], byte[]> producer = Support.newProducer(badAuth)) {
            Exception ex = assertThrows(Exception.class,
                    () -> producer.send(new ProducerRecord<>(topic, 0, null, "x".getBytes())).get());
            Throwable cause = (ex instanceof ExecutionException) ? ex.getCause() : ex;
            // Chase down the chain: a bad SCRAM password surfaces as SaslAuthenticationException.
            boolean sawSaslFailure = false;
            for (Throwable t = cause; t != null; t = t.getCause()) {
                if (t instanceof SaslAuthenticationException) {
                    sawSaslFailure = true;
                    break;
                }
            }
            assertTrue(sawSaslFailure, "a wrong password must fail SASL authentication, got " + cause);
        }
    }
}
