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
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ExecutionException;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.common.errors.InvalidRequestException;
import org.apache.kafka.common.quota.ClientQuotaAlteration;
import org.apache.kafka.common.quota.ClientQuotaEntity;
import org.apache.kafka.common.quota.ClientQuotaFilter;
import org.apache.kafka.common.quota.ClientQuotaFilterComponent;
import org.junit.jupiter.api.Test;

/**
 * Phase 4a: client quotas — AlterClientQuotas(49), DescribeClientQuotas(48).
 * Only the {@code client-id} entity and producer/consumer byte-rate keys are
 * supported.
 */
class ClientQuotaTest {

    private static final String PRODUCER_RATE = "producer_byte_rate";

    private static String clientId() {
        return "svc-" + UUID.randomUUID();
    }

    private static ClientQuotaEntity clientEntity(String id) {
        return new ClientQuotaEntity(Map.of(ClientQuotaEntity.CLIENT_ID, id));
    }

    private static Map<String, Double> describe(Admin admin, String id) throws Exception {
        ClientQuotaFilter filter = ClientQuotaFilter.contains(
                List.of(ClientQuotaFilterComponent.ofEntity(ClientQuotaEntity.CLIENT_ID, id)));
        Map<ClientQuotaEntity, Map<String, Double>> result =
                admin.describeClientQuotas(filter).entities().get();
        return result.getOrDefault(clientEntity(id), Map.of());
    }

    @Test
    void setDescribeAndRemoveClientQuota() throws Exception {
        String id = clientId();
        ClientQuotaEntity entity = clientEntity(id);
        try (Admin admin = Support.newAdmin()) {
            admin.alterClientQuotas(List.of(new ClientQuotaAlteration(entity,
                    List.of(new ClientQuotaAlteration.Op(PRODUCER_RATE, 1048576.0))))).all().get();

            assertEquals(1048576.0, describe(admin, id).get(PRODUCER_RATE),
                    "the set quota should be described");

            // A null value removes the quota.
            admin.alterClientQuotas(List.of(new ClientQuotaAlteration(entity,
                    List.of(new ClientQuotaAlteration.Op(PRODUCER_RATE, null))))).all().get();

            assertTrue(describe(admin, id).get(PRODUCER_RATE) == null,
                    "the removed quota should no longer be described");
        }
    }

    @Test
    void unsupportedUserEntityIsRejected() throws Exception {
        try (Admin admin = Support.newAdmin()) {
            ClientQuotaEntity userEntity =
                    new ClientQuotaEntity(Map.of(ClientQuotaEntity.USER, "alice"));
            ExecutionException ex = assertThrows(ExecutionException.class,
                    () -> admin.alterClientQuotas(List.of(new ClientQuotaAlteration(userEntity,
                            List.of(new ClientQuotaAlteration.Op(PRODUCER_RATE, 1000.0)))))
                            .all().get());
            assertInstanceOf(InvalidRequestException.class, ex.getCause(),
                    "only the client-id entity type is supported");
        }
    }
}
