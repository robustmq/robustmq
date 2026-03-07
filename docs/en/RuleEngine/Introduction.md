# Rule Engine Introduction

## What is Connector

A Connector is the data ingress/egress adapter layer in RobustMQ.
It is responsible for delivering messages to target systems (Kafka, MySQL, Redis, Webhook, etc.) or pulling data from external systems and writing it back into the message pipeline.

In short, Connector answers:

- where data should go
- how data is transported
- how target integration is implemented

## What is Rule Engine

The Rule Engine is the data-processing layer in the connector pipeline.
Before data is sent to the target system, it applies lightweight, configurable, and testable transformations.

Current runtime model:

- `decode`: normalize source payload into a unified structure
- `ops[]`: run ordered operators (e.g. `Extract`)
- `encode`: serialize transformed records into output bytes

## Relationship Between Connector and Rule Engine

They are complementary:

- **Connector** decides destination and delivery
- **Rule Engine** decides transformation before delivery

End-to-end pipeline:

`message input -> decode -> ops[] -> encode -> connector sink`

## What It Is Used For

This design solves three practical problems:

- lower integration cost for common transformations
- unified processing model across heterogeneous sources and sinks
- better operability through explicit rules and reproducible behavior

## Boundary

The Rule Engine focuses on lightweight stateless processing.
It does not aim to replace stateful stream frameworks (window aggregation, cross-message joins, etc.).

This boundary keeps the broker/connector path stable while covering most high-frequency data-cleaning scenarios.
