# MQTT Flapping Detection

## What is Flapping Detection?

Flapping Detection is a security feature provided by RobustMQ to automatically detect and ban clients that frequently connect and disconnect within a short period of time. This mechanism effectively prevents malicious clients or misconfigured clients from excessively consuming server resources, ensuring normal usage for other clients.

## When to Use Flapping Detection?

Flapping detection is particularly useful in scenarios where:

- **Prevent Malicious Attacks**: Block malicious clients from consuming server resources through frequent connections
- **Protect System Stability**: Prevent misconfigured clients from affecting system performance
- **Resource Management**: Ensure server resources are fairly allocated to all clients
- **Security Protection**: As part of the security protection system, improve system security

## Features of Flapping Detection

### Automatic Detection Mechanism

RobustMQ's flapping detection feature provides:

- **Intelligent Monitoring**: Automatically monitor client connection behavior patterns
- **Dynamic Banning**: Automatically ban clients when abnormal behavior is detected
- **Time Window Control**: Count connection attempts within specified time windows
- **Flexible Configuration**: Support custom detection parameters and ban policies

### Key Characteristics

- **Client ID Banning**: Only bans client IDs, not usernames and IP addresses
- **Temporary Banning**: Bans are temporary and automatically lifted when expired
- **Configurable Parameters**: Support custom detection time windows, maximum disconnection counts, and ban duration
- **Real-time Effect**: Configuration changes take effect immediately

## Flapping Detection Configuration

### Configure via RobustMQ Dashboard

1. Access RobustMQ Dashboard: [http://117.72.92.117:8080/](http://117.72.92.117:8080/)
2. Navigate to **Access Control** -> **Flapping Detection**
3. Enable flapping detection feature
4. Configure the following parameters:

   - **Detection Time Window**: Duration for monitoring client flapping behavior (default: 1 minute)
   - **Maximum Disconnection Count**: Maximum allowed disconnection count within the detection window (default: 15 times)
   - **Ban Duration**: Time length for client banning (default: 5 minutes)

5. Click **Save Changes** to complete configuration

### Configure via Configuration File

Add the following configuration to the RobustMQ configuration file:

```toml
[flapping_detect]
enable = true
# Maximum offline count for clients
max_count = 15
# Detection time range
window_time = "1m"
# Ban duration
ban_time = "5m"
```

This configuration means that when a client accumulates 15 disconnection times within 1 minute, it will be banned for 5 minutes.

## How Flapping Detection Works

### Detection Process

1. **Behavior Monitoring**: System continuously monitors connection and disconnection behavior of all clients
2. **Count Statistics**: Count disconnection times for each client within the configured time window
3. **Threshold Judgment**: When disconnection count exceeds the configured maximum value, trigger the ban mechanism
4. **Automatic Banning**: Add abnormal client IDs to temporary blacklist
5. **Automatic Removal**: Automatically remove from blacklist when ban time expires

### Banning Mechanism

- **Ban Scope**: Only bans client IDs, does not affect usernames and IP addresses
- **Ban Effect**: Banned clients cannot establish new connections
- **Bypass Method**: Clients can bypass bans by changing client IDs
- **Automatic Recovery**: Automatically restore connection permissions when ban time expires

## Practical Application Examples

### Scenario 1: Prevent Malicious Attacks

```bash
# Malicious client frequently connecting
mqttx conn -i malicious_client -h '117.72.92.117' -p 1883
# Quickly disconnect
# Repeat the above operation multiple times

# System detects abnormal behavior and automatically bans the client ID
# Client receives connection refused error
```

### Scenario 2: Misconfigured Client

```bash
# Misconfigured client (e.g., wrong authentication credentials)
mqttx conn -i bad_config_client -u wrong_user -P wrong_pass -h '117.72.92.117' -p 1883
# Connection fails, client retries
# After multiple repetitions, triggers flapping detection

# System bans the client ID to prevent continued resource consumption
```

### Scenario 3: Normal Clients Unaffected

```bash
# Normal client connection
mqttx conn -i normal_client -h '117.72.92.117' -p 1883
# Normal usage, will not trigger flapping detection
# Even occasional disconnection and reconnection will not be mistakenly banned
```

## Configuration Parameters

### Detection Time Window (window_time)

- **Purpose**: Define the time range for monitoring client behavior
- **Default Value**: 1 minute
- **Recommended Value**: Adjust according to business scenarios, usually set to 1-5 minutes
- **Impact**: Longer time window means more lenient detection; shorter time window means stricter detection

### Maximum Disconnection Count (max_count)

- **Purpose**: Maximum allowed disconnection count within the time window
- **Default Value**: 15 times
- **Recommended Value**: Adjust according to normal client reconnection frequency
- **Impact**: Smaller value means stricter detection; larger value means more lenient detection

### Ban Duration (ban_time)

- **Purpose**: Time length for client banning
- **Default Value**: 5 minutes
- **Recommended Value**: Adjust according to security requirements, usually set to 5-30 minutes
- **Impact**: Longer ban time means stronger security protection, but may affect quick recovery of normal clients

## Monitoring and Management

### Monitor via Dashboard

1. Access RobustMQ Dashboard
2. View **Access Control** -> **Flapping Detection** page
3. Monitor the following information:
   - Current banned client list
   - Ban reasons and remaining time
   - Detection statistics

### Log Monitoring

Flapping detection related logs are recorded in system logs, including:

- Client ban events
- Ban removal events
- Detection statistics

## Important Notes

- **Client ID Uniqueness**: Ensure each client uses a unique client ID
- **Reasonable Parameter Configuration**: Adjust detection parameters according to actual business scenarios
- **Monitor System Status**: Regularly check ban lists and system logs
- **Test Environment Validation**: Validate configuration in test environment before production deployment
- **Backup Configuration**: Save configuration file backups for quick recovery
