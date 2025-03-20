## Overview

Deferred publication is an MQTT extension supported by RobustMQ MQTT. When the client publishes a message using the special topic prefix $delayed/{DelayInterval}, the delayed publication feature is triggered, which allows the user to delay the publication of the message at a user-configured interval.

## Functional Description
The format of a deferred publication topic is as follows:
```
$delayed/{DelayInterval}/{TopicName}
```
- $delayed：Any message that uses $delay as the topic prefix will be treated as a message that needs to be delayed. The delay interval is determined by the content in the next topic level.
- {DelayInterval}：Specifies the time interval, in seconds, at which the MQTT message is delayed, with the maximum allowed interval being 4294967 seconds. If the {DelayInterval} cannot be resolved to an integer number, EMQX drops the message and the client receives no information.
- {TopicName}：The topic name of the MQTT message.

## Examples
- $delayed/15/x/y：The MQTT message is published to topic x/y after 15 seconds.
- $delayed/60/a/b：The MQTT message is published to a/b after 1 minute.
- $delayed/3600/$SYS/topic：Post the MQTT message to $SYS/topic after 1 hour.
