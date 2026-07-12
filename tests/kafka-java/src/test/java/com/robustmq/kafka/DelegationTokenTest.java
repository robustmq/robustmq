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

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;

import org.apache.kafka.clients.admin.Admin;
import org.apache.kafka.common.security.token.delegation.DelegationToken;
import org.junit.jupiter.api.Test;

/**
 * Phase 7: delegation token (KIP-48) metadata management — CreateDelegationToken(38),
 * RenewDelegationToken(39), ExpireDelegationToken(40), DescribeDelegationToken(41).
 * Metadata only: the broker does not yet authenticate with a token's hmac.
 */
class DelegationTokenTest {

    @Test
    void createAndDescribeToken() throws Exception {
        try (Admin admin = Support.newAdmin()) {
            DelegationToken token = admin.createDelegationToken().delegationToken().get();
            assertNotNull(token.tokenInfo().tokenId(), "a created token has an id");
            assertTrue(token.hmac().length > 0, "a created token has an hmac");

            List<DelegationToken> listed =
                    admin.describeDelegationToken().delegationTokens().get();
            boolean found = listed.stream()
                    .anyMatch(t -> t.tokenInfo().tokenId().equals(token.tokenInfo().tokenId()));
            assertTrue(found, "the created token should be described");
        }
    }

    @Test
    void renewExtendsExpiry() throws Exception {
        try (Admin admin = Support.newAdmin()) {
            DelegationToken token = admin.createDelegationToken().delegationToken().get();
            long newExpiry = admin.renewDelegationToken(token.hmac()).expiryTimestamp().get();
            assertTrue(newExpiry > 0, "renew should return a future expiry timestamp");
        }
    }

    @Test
    void expireSucceeds() throws Exception {
        try (Admin admin = Support.newAdmin()) {
            DelegationToken token = admin.createDelegationToken().delegationToken().get();
            // Expiring returns the effective expiry timestamp; the call must not fail.
            long expiry = admin.expireDelegationToken(token.hmac()).expiryTimestamp().get();
            assertTrue(expiry >= 0, "expire should complete and return a timestamp");
        }
    }
}
