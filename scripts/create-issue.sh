#!/bin/bash

# Script to create GitHub issues using GitHub API
# Usage: ./scripts/create-issue.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if GITHUB_TOKEN is set
if [ -z "$GITHUB_TOKEN" ]; then
    echo -e "${RED}Error: GITHUB_TOKEN environment variable is not set${NC}"
    echo "Please set it with: export GITHUB_TOKEN=your_github_token"
    exit 1
fi

REPO="robustmq/robustmq"

# Issue details
TITLE="[BUG] Memory Leak in Retain Message Cache - Expired Messages Not Cleaned Up"

read -r -d '' BODY << 'EOFBODY' || true
## 📋 Summary

The \`RetainMessageManager\` in \`src/mqtt-broker/src/core/retain.rs\` has a memory leak issue where expired retain messages are detected but not removed from the cache, causing memory to grow indefinitely over time.

## 🔍 Current Behavior

When sending a retain message, the code checks if the message has expired:

\`\`\`rust
// In send_retain_message() - line 251-253
if now_second() >= *message_at {
    // Message has expired
    return Ok(());
}
\`\`\`

**When a message expires:**
1. The code detects expiration and skips sending
2. Error/warning is NOT logged
3. Message remains in \`topic_retain_data\` cache indefinitely
4. Message remains in \`last_update_time\` cache indefinitely
5. No cleanup, no metrics update
6. ⚠️ **Memory keeps growing with each expired message**

**Common scenarios:**
- Messages with short expiry times (e.g., 10-60 seconds)
- High-volume retain message publishing
- Long-running broker instances
- Messages with \`message_expiry_interval\` property

## ✨ Expected Behavior

When a retain message expires, the system should:

1. Remove it from \`topic_retain_data\` cache
2. Remove it from \`last_update_time\` cache
3. Delete it from persistent storage
4. Update metrics (\`record_mqtt_retained_dec()\`)
5. Log the cleanup action

## 🐛 Root Cause

**Location:** \`src/mqtt-broker/src/core/retain.rs\`, line 251-253

The \`send_retain_message()\` method detects expired messages but only returns early without cleanup:

\`\`\`rust
if now_second() >= *message_at {
    // Message has expired
    return Ok(());  // ❌ Just returns, no cleanup!
}
\`\`\`

## 💡 Proposed Solution

Add cleanup logic when an expired message is detected:

\`\`\`rust
if now_second() >= *message_at {
    // Message has expired - clean it up
    drop(cache_entry); // Release the lock first
    
    // Remove from caches
    self.topic_retain_data.remove(&data.topic_name);
    self.last_update_time.remove(&data.topic_name);
    
    // Remove from storage and update metrics
    let topic_storage = TopicStorage::new(self.client_pool.clone());
    if let Ok(_) = topic_storage.delete_retain_message(&data.topic_name).await {
        record_mqtt_retained_dec();
        debug!(
            "Expired retain message cleaned up: topic={}",
            data.topic_name
        );
    }
    
    return Ok(());
}
\`\`\`

### Alternative: Background Cleanup Task

For proactive cleanup, consider adding a periodic task:

\`\`\`rust
pub fn start_cleanup_expired_retain_messages(
    retain_message_manager: Arc<RetainMessageManager>,
    cleanup_interval_secs: u64,
) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(
            Duration::from_secs(cleanup_interval_secs)
        );
        
        loop {
            interval.tick().await;
            
            let now = now_second();
            let mut expired_topics = Vec::new();
            
            // Find expired messages
            for entry in retain_message_manager.topic_retain_data.iter() {
                if let Some((_, expiry_time)) = entry.value() {
                    if now >= *expiry_time {
                        expired_topics.push(entry.key().clone());
                    }
                }
            }
            
            // Clean up expired messages
            for topic in expired_topics {
                // Remove from caches and storage
                // Update metrics
            }
        }
    });
}
\`\`\`

## 📍 Location

- **File**: \`src/mqtt-broker/src/core/retain.rs\`
- **Method**: \`send_retain_message()\` (line 231-338)
- **Problem Line**: 251-253

## 📊 Impact

**Severity:** Medium to High

**Affected Scenarios:**
- Brokers handling retain messages with expiry
- Long-running production deployments
- High-volume message publishing

**Memory Growth:**
- Linear growth: +1 entry per expired message
- Each entry: \`MqttMessage\` object + metadata (can be KB to MB per message)
- No automatic cleanup = unbounded growth

**Side Effects:**
1. **Memory bloat**: Accumulation of expired messages
2. **Performance degradation**: Slower cache lookups with more entries
3. **Inaccurate metrics**: \`contain_retain()\` returns true for expired messages
4. **Potential OOM**: In extreme cases, could lead to out-of-memory errors

## 🧪 Test Scenario

**Reproduce Steps:**
1. Start MQTT broker
2. Publish retain messages with short expiry (e.g., 10 seconds):
   \`\`\`
   PUBLISH topic="test/retain" retain=true message_expiry_interval=10
   \`\`\`
3. Wait for messages to expire (>10 seconds)
4. Monitor memory usage with \`ps\`, \`top\`, or Prometheus metrics
5. Repeat steps 2-4 multiple times

**Expected:** Memory usage should decrease after expiry
**Actual:** Memory continuously grows

## 🔗 Related Code

- \`save_retain_message()\`: Sets retain messages and updates cache
- \`contain_retain()\`: Checks message existence (doesn't filter expired)
- \`load_retain_from_storage()\`: Loads messages into cache
- \`start_send_retain_thread()\`: Background worker that calls \`send_retain_message()\`

## ✅ Acceptance Criteria

- [ ] Add cleanup logic in \`send_retain_message()\` when message expires
- [ ] Remove expired message from \`topic_retain_data\` cache
- [ ] Remove expired message from \`last_update_time\` cache
- [ ] Delete expired message from persistent storage
- [ ] Update metrics (\`record_mqtt_retained_dec()\`)
- [ ] Add debug log for cleanup actions
- [ ] (Optional) Implement background cleanup task for proactive cleanup
- [ ] Add configuration for cleanup interval
- [ ] Add unit tests for expiry cleanup
- [ ] Add integration test with expired messages
- [ ] Update documentation

## 🔧 Additional Considerations

1. **Concurrency**: Ensure cleanup is thread-safe with DashMap operations
2. **Storage sync**: Verify storage deletion doesn't fail silently
3. **Metrics accuracy**: Ensure counter doesn't go negative
4. **Config**: Consider making cleanup behavior configurable
5. **Logging**: Use appropriate log levels (debug for normal cleanup, warn for errors)

## 🎯 Priority

**High** - Causes memory leak in production environments with message expiry

## 📦 Environment

- **Component**: MQTT Broker
- **Module**: Retain Message Manager
- **File**: \`src/mqtt-broker/src/core/retain.rs\`
EOFBODY

LABELS='["kind:bug", "mqtt", "priority:high", "memory-leak"]'

# Create JSON payload
JSON_PAYLOAD=$(jq -n \
    --arg title "$TITLE" \
    --arg body "$BODY" \
    --argjson labels "$LABELS" \
    '{title: $title, body: $body, labels: $labels}')

echo -e "${YELLOW}Creating GitHub issue...${NC}"
echo -e "Repository: ${GREEN}$REPO${NC}"
echo -e "Title: ${GREEN}$TITLE${NC}"
echo ""

# Create the issue
RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST \
    -H "Accept: application/vnd.github.v3+json" \
    -H "Authorization: token $GITHUB_TOKEN" \
    "https://api.github.com/repos/$REPO/issues" \
    -d "$JSON_PAYLOAD")

HTTP_CODE=$(echo "$RESPONSE" | tail -n1)
RESPONSE_BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "201" ]; then
    ISSUE_URL=$(echo "$RESPONSE_BODY" | jq -r '.html_url')
    ISSUE_NUMBER=$(echo "$RESPONSE_BODY" | jq -r '.number')
    echo -e "${GREEN}✓ Issue created successfully!${NC}"
    echo -e "Issue #${ISSUE_NUMBER}: ${ISSUE_URL}"
else
    echo -e "${RED}✗ Failed to create issue${NC}"
    echo -e "HTTP Status: $HTTP_CODE"
    echo "$RESPONSE_BODY" | jq '.'
    exit 1
fi
