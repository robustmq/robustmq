## Topic Rewrite
Topic Rewrite allows users to configure rules to change the topic strings that
the client requests to subscribe or publish.

This feature is very useful when there's a need to transform between different topic structures.
For example, an old device that has already been issued and cannot
be upgraded may use old topic designs, but for some reason, we adjusted the format of topics. We can use this feature 
to rewrite the old topics as the new format to eliminate these differences.

More introduction about [Topic Rewrite](https://www.emqx.io/docs/en/v5.0/mqtt/mqtt-topic-rewrite.html).

See [List all rewrite rules](https://www.emqx.io/docs/en/v5.0/admin/api-docs.html#tag/MQTT/paths/~1mqtt~1topic_rewrite/get)
and [Create or Update rewrite rules](https://www.emqx.io/docs/en/v5.0/admin/api-docs.html#tag/MQTT/paths/~1mqtt~1topic_rewrite/put).

### Configure Topic Rewrite Rules
RobustMQ topic rewrite rules are configured by the placement amin service. You may add multiple topic rewrite rules. The number of rules is 
not limited, but any MQTT message that carries a topic needs to match the rewrite rule again. Therefore, 
the performance overhead in high-throughput scenarios is proportional to the number of rules, 
so, use this feature with caution.

The request format of rewrite rule for each topic is as follows:
```rust
let action: String = "All".to_string();
let source_topic: String = "x/#".to_string();
let dest_topic: String = "x/y/z/$1".to_string();
let re: String = "^x/y/(.+)$".to_string();

let req = CreateTopicRewriteRuleRequest {
    action: action.clone(),
    source_topic: source_topic.clone(),
    dest_topic: dest_topic.clone(),
    regex: re.clone(),
};
```
Each rewrite rule consists of a filter, regular expression.

The rewrite rules are divided into publish, subscribe and all rules. The publish rule matches the topics carried 
by PUBLISH messages, and the subscribe rule matches the topics carried by SUBSCRIBE and UNSUBSCRIBE messages. 
The all rule is valid for topics carried by PUBLISH, SUBSCRIBE and UNSUBSCRIBE messages.

On the premise that the topic rewrite is enabled, when receiving MQTT packet such as PUBLISH messages with a topic,
RobustMQ will use the topic in the packet to sequentially match the topic filter part of the rules in the configuration 
file. Once the match is successful the regular expression is used to extract the information in the topic, 
and then the old topic is replaced by the target expression to form a new topic.

The target expression can use variables in the format of $N to match the elements extracted from the regular expression.
The value of $N is the Nth element extracted from the regular expression, for example, $1 is the first element 
extracted by the regular expression.

And the target expression alose support use ${clientid} to represent the client ID and ${username} to represent the client username.

It should be noted that RobustMQ reads the rewrite rules in order of the configuration file. When a topic can match the 
topic filters of multiple topic rewrite rules at the same time, RobustMQ uses the first matching rule to rewrite the topic.

If the regular expression in the rule does not match the topic of the MQTT packet, the rewrite fails, and no other 
rule will be used to rewrite. Therefore, users need to carefully design MQTT packet topics and topic rewrite rules.