# Optimize Subscribe/Unsubscribe Operations with Batch API Support

## Summary

Currently, `placement_set_subscribe` and `placement_delete_subscribe` process subscriptions one at a time. When a client sends a SUBSCRIBE/UNSUBSCRIBE packet with multiple topic filters, the broker makes multiple sequential gRPC calls to the meta-service, which is inefficient and can lead to partial failures without atomicity guarantees.

## Motivation

### Current Implementation Issues

1. **Multiple gRPC Calls**: For a SUBSCRIBE packet with N topic filters, the broker makes N separate gRPC calls to `placement_set_subscribe`
2. **Network Overhead**: Each gRPC call has network latency and serialization overhead
3. **Partial Failures**: If the 3rd filter fails out of 5, the first 2 are already persisted, causing inconsistent state
4. **No Atomicity**: No transaction support - can't rollback on failure

### Example Scenario

```rust
// Current: N gRPC calls for N filters
SUBSCRIBE {
    filters: [
        "sensor/temp" → placement_set_subscribe() call 1
        "sensor/humidity" → placement_set_subscribe() call 2
        "device/#" → placement_set_subscribe() call 3
    ]
}

// If call 3 fails:
// - Meta service has 2 subscriptions saved
// - Client receives SUBACK with all failed reason codes
// - Inconsistent state between client expectation and server state
```

## Proposed Solution

### Add Batch APIs

Add new gRPC methods for batch operations:

```protobuf
// meta_service_mqtt.proto

message SetSubscribeBatchRequest {
    string client_id = 1;
    repeated SubscribeItem items = 2;
}

message SubscribeItem {
    string path = 1;
    bytes subscribe = 2;  // serialized MqttSubscribe
}

message SetSubscribeBatchResponse {
    repeated SubscribeResult results = 1;
}

message SubscribeResult {
    string path = 1;
    bool success = 2;
    string error = 3;  // optional error message
}

message DeleteSubscribeBatchRequest {
    string client_id = 1;
    repeated string paths = 2;
}

message DeleteSubscribeBatchResponse {
    repeated SubscribeResult results = 1;
}

service MqttService {
    rpc SetSubscribeBatch(SetSubscribeBatchRequest) returns (SetSubscribeBatchResponse);
    rpc DeleteSubscribeBatch(DeleteSubscribeBatchRequest) returns (DeleteSubscribeBatchResponse);
}
```

### Updated Implementation

```rust
// src/mqtt-broker/src/core/subscribe.rs

pub async fn save_subscribe(context: SaveSubscribeContext) -> ResultMqttBrokerError {
    let conf = broker_config();
    let filters = &context.subscribe.filters;
    
    // Build batch request
    let items: Vec<SubscribeItem> = filters.iter().map(|filter| {
        let subscribe_data = MqttSubscribe {
            client_id: context.client_id.clone(),
            path: filter.path.clone(),
            broker_id: conf.broker_id,
            filter: filter.clone(),
            pkid: context.subscribe.packet_identifier,
            subscribe_properties: context.subscribe_properties.clone(),
            protocol: context.protocol.clone(),
            create_time: now_second(),
        };
        
        SubscribeItem {
            path: filter.path.clone(),
            subscribe: subscribe_data.encode()?,
        }
    }).collect()?;
    
    let request = SetSubscribeBatchRequest {
        client_id: context.client_id.clone(),
        items,
    };
    
    // Single gRPC call for all filters
    let response = placement_set_subscribe_batch(
        &context.client_pool,
        &conf.get_meta_service_addr(),
        request
    ).await?;
    
    // Handle partial failures
    for result in response.results {
        if !result.success {
            error!("Failed to save subscription for path {}: {}", result.path, result.error);
            // Return error or collect all failures
        }
    }
    
    Ok(())
}
```

## Benefits

1. **Performance**: 1 gRPC call instead of N calls
2. **Network Efficiency**: Reduced network round-trips and overhead
3. **Better Error Handling**: Atomic batch operations with clear per-item results
4. **Consistency**: All-or-nothing semantics (optional with transaction support)
5. **Scalability**: Better performance under high load with many subscriptions

## Implementation Checklist

- [ ] Add protobuf definitions for batch APIs
- [ ] Implement `SetSubscribeBatch` in meta-service
- [ ] Implement `DeleteSubscribeBatch` in meta-service
- [ ] Add gRPC client wrappers in `grpc-clients`
- [ ] Update `save_subscribe()` to use batch API
- [ ] Update `remove_subscribe()` to use batch API
- [ ] Add transaction support for atomicity (optional)
- [ ] Add unit tests for batch operations
- [ ] Add integration tests with partial failures
- [ ] Update documentation

## Performance Impact

**Before**: 10 filters = 10 gRPC calls ≈ 10ms * 10 = 100ms

**After**: 10 filters = 1 gRPC call ≈ 12ms (bulk processing overhead)

**Improvement**: ~8x faster for 10 filters

## Related Issues

- Subscription persistence across broker restarts
- Cross-node subscription restoration on client reconnection
- MQTT protocol compliance for SUBSCRIBE/UNSUBSCRIBE packets
