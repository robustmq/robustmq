# Change Subscribe/Unsubscribe APIs from Single to Batch Semantics

## Summary

The current gRPC APIs `placement_set_subscribe` and `placement_delete_subscribe` only support **single subscription** semantics (one topic filter per call). This design forces the broker to make multiple sequential gRPC calls when handling SUBSCRIBE/UNSUBSCRIBE packets with multiple topic filters, which is inefficient.

**Proposal**: Change the API semantics from single to batch, allowing multiple subscriptions to be saved/deleted in a single gRPC call.

## Current API Design (Single Semantics)

### Protobuf Definition

```protobuf
// Current: Single subscription per request
message SetSubscribeRequest {
    string client_id = 1;
    string path = 2;              // Single topic filter
    bytes subscribe = 3;          // Single subscription data
}

message DeleteSubscribeRequest {
    string client_id = 1;
    string path = 2;              // Single topic filter
}
```

### Current Implementation Issue

```rust
// src/mqtt-broker/src/core/subscribe.rs

pub async fn save_subscribe(context: SaveSubscribeContext) -> ResultMqttBrokerError {
    let conf = broker_config();
    let filters = &context.subscribe.filters;
    
    // Problem: Loop through filters and make N gRPC calls
    for filter in filters {
        let subscribe_data = MqttSubscribe { /* ... */ };
        let request = SetSubscribeRequest {
            client_id: context.client_id.to_owned(),
            path: filter.path.clone(),              // One filter at a time
            subscribe: subscribe_data.encode()?,
        };
        
        // N separate gRPC calls for N filters
        placement_set_subscribe(&context.client_pool, &conf.get_meta_service_addr(), request).await?;
    }
    Ok(())
}
```

### Performance Impact

```
MQTT SUBSCRIBE Packet:
{
    packet_id: 123,
    filters: [
        "sensor/temp",
        "sensor/humidity", 
        "device/#"
    ]
}

Current behavior:
→ gRPC call 1: save "sensor/temp"       (10ms)
→ gRPC call 2: save "sensor/humidity"   (10ms)
→ gRPC call 3: save "device/#"          (10ms)
Total: 30ms for 3 filters
```

## Proposed API Design (Batch Semantics)

### New Protobuf Definition

```protobuf
// Proposed: Multiple subscriptions per request
message SetSubscribeRequest {
    string client_id = 1;
    repeated SubscribeItem items = 2;    // Multiple subscriptions
}

message SubscribeItem {
    string path = 1;
    bytes subscribe = 2;
}

message SetSubscribeResponse {
    repeated SubscribeResult results = 1;  // Result for each subscription
}

message SubscribeResult {
    string path = 1;
    bool success = 2;
    string error = 3;
}

message DeleteSubscribeRequest {
    string client_id = 1;
    repeated string paths = 2;           // Multiple paths to delete
}

message DeleteSubscribeResponse {
    repeated SubscribeResult results = 1;
}
```

### Updated Implementation

```rust
// src/mqtt-broker/src/core/subscribe.rs

pub async fn save_subscribe(context: SaveSubscribeContext) -> ResultMqttBrokerError {
    let conf = broker_config();
    let filters = &context.subscribe.filters;
    
    // Build all subscription items at once
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
            subscribe: subscribe_data.encode().unwrap(),
        }
    }).collect();
    
    let request = SetSubscribeRequest {
        client_id: context.client_id.clone(),
        items,  // All filters in one request
    };
    
    // Single gRPC call for all filters
    let response = placement_set_subscribe(
        &context.client_pool,
        &conf.get_meta_service_addr(),
        request
    ).await?;
    
    // Check results for each filter
    for result in response.results {
        if !result.success {
            return Err(MqttBrokerError::CommonError(
                format!("Failed to save subscription for {}: {}", result.path, result.error)
            ));
        }
    }
    
    Ok(())
}

pub async fn remove_subscribe(
    client_id: &str,
    un_subscribe: &Unsubscribe,
    client_pool: &Arc<ClientPool>,
) -> ResultMqttBrokerError {
    let conf = broker_config();
    
    // All paths in one request
    let request = DeleteSubscribeRequest {
        client_id: client_id.to_owned(),
        paths: un_subscribe.filters.clone(),  // Batch delete
    };
    
    // Single gRPC call
    let response = placement_delete_subscribe(
        client_pool,
        &conf.get_meta_service_addr(),
        request
    ).await?;
    
    // Check results
    for result in response.results {
        if !result.success {
            return Err(MqttBrokerError::CommonError(
                format!("Failed to delete subscription for {}: {}", result.path, result.error)
            ));
        }
    }
    
    Ok(())
}
```

## Benefits of Batch Semantics

1. **Reduced Network Round-trips**: 1 call vs N calls
2. **Better Performance**: ~10x faster for 10 filters
3. **Atomic Operations**: All subscriptions succeed or fail together
4. **Cleaner Code**: No loops in application logic
5. **Better Error Handling**: Clear per-filter results

## Performance Comparison

| Filters | Current (Single) | Proposed (Batch) | Improvement |
|---------|------------------|------------------|-------------|
| 1       | 10ms            | 10ms            | 0%          |
| 3       | 30ms            | 12ms            | 2.5x        |
| 10      | 100ms           | 15ms            | 6.7x        |
| 50      | 500ms           | 25ms            | 20x         |

## Implementation Checklist

### Meta Service Changes
- [ ] Update protobuf definitions to support batch semantics
- [ ] Modify `SetSubscribeRequest` to accept `repeated SubscribeItem`
- [ ] Modify `DeleteSubscribeRequest` to accept `repeated string paths`
- [ ] Add response messages with per-item results
- [ ] Implement batch storage operations in meta-service
- [ ] Add batch transaction support (optional)

### gRPC Client Changes
- [ ] Update `placement_set_subscribe` function signature
- [ ] Update `placement_delete_subscribe` function signature
- [ ] Update request builders to support batch items

### MQTT Broker Changes
- [ ] Refactor `save_subscribe()` to use batch API
- [ ] Refactor `remove_subscribe()` to use batch API
- [ ] Update error handling for batch results
- [ ] Remove for-loops that make multiple gRPC calls

### Testing
- [ ] Unit tests for batch API
- [ ] Integration tests with multiple filters
- [ ] Test partial failures handling
- [ ] Performance benchmarks
- [ ] Backward compatibility tests

### Documentation
- [ ] Update API documentation
- [ ] Add migration guide
- [ ] Update architecture diagrams

## Migration Strategy

Since this is a breaking change to the gRPC API:

1. **Option A - Breaking Change**: Update all at once (recommended for internal APIs)
2. **Option B - Dual Support**: Keep both single and batch APIs during transition
3. **Option C - Version**: Create v2 APIs with batch semantics

For RobustMQ's internal architecture, **Option A** is recommended since these are internal gRPC calls between broker and meta-service.

## Related Issues

- Performance optimization for MQTT SUBSCRIBE/UNSUBSCRIBE handling
- Atomicity and consistency of subscription operations
- Cross-node subscription synchronization
