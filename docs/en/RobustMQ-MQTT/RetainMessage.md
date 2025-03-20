## What are MQTT reserved messages?
When a publisher publishes a Message, it is a Retained Message in MQTT if the Retained flag is set to true. The MQTT server stores the latest reserved message for each topic, so that clients that are online after the message is published can still receive the message when they subscribe to a topic. When a client subscribes to a topic, if there exists a reserved message matching the topic on the server, the reserved message will be sent to the client immediately.

## When to use MQTT to retain messages?
Although the publishing-subscribe model can decouple the publisher from the subscriber, it also has the disadvantage that the subscriber cannot request messages from the publisher. When the subscriber receives the message is completely dependent on when the publisher publishes the message, which is inconvenient in some scenarios.

With reservation messages, new subscribers are able to get the most recent status immediately, without having to wait an unpredictable amount of time, such as:
- The status of smart home devices will only be reported when they change, but the controller needs to be able to access the status of the device after it is online;
- The interval between sensors reporting data is too long, but the subscriber needs to get the latest data immediately after subscribing;
- Sensor version numbers, serial numbers, and other attributes that do not change frequently can be published as a reserved message to all subsequent subscribers after being online.