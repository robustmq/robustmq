#!/bin/bash

# Test script for blacklist create API parameter validation
# Make sure the RobustMQ server is running before executing this script

BASE_URL="http://localhost:8030/api"

echo "======================================"
echo "Testing Blacklist Create API Validation"
echo "======================================"
echo ""

# Test 1: Valid request
echo "Test 1: Valid request (should succeed)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "ClientId",
    "resource_name": "test_client_001",
    "end_time": 9999999999,
    "desc": "Test blacklist entry"
  }' | jq .
echo -e "\n"

# Test 2: Invalid blacklist_type (empty)
echo "Test 2: Invalid blacklist_type (empty, should fail)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "",
    "resource_name": "test_client",
    "end_time": 9999999999,
    "desc": "Test"
  }' | jq .
echo -e "\n"

# Test 3: Invalid blacklist_type (wrong value)
echo "Test 3: Invalid blacklist_type (InvalidType, should fail)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "InvalidType",
    "resource_name": "test_client",
    "end_time": 9999999999,
    "desc": "Test"
  }' | jq .
echo -e "\n"

# Test 4: Invalid resource_name (empty)
echo "Test 4: Invalid resource_name (empty, should fail)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "ClientId",
    "resource_name": "",
    "end_time": 9999999999,
    "desc": "Test"
  }' | jq .
echo -e "\n"

# Test 5: Invalid resource_name (too long)
echo "Test 5: Invalid resource_name (too long > 256 chars, should fail)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d "{
    \"blacklist_type\": \"ClientId\",
    \"resource_name\": \"$(python3 -c 'print("a" * 257)')\",
    \"end_time\": 9999999999,
    \"desc\": \"Test\"
  }" | jq .
echo -e "\n"

# Test 6: Invalid end_time (0)
echo "Test 6: Invalid end_time (0, should fail)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "ClientId",
    "resource_name": "test_client",
    "end_time": 0,
    "desc": "Test"
  }' | jq .
echo -e "\n"

# Test 7: Invalid desc (too long)
echo "Test 7: Invalid desc (too long > 500 chars, should fail)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d "{
    \"blacklist_type\": \"ClientId\",
    \"resource_name\": \"test_client\",
    \"end_time\": 9999999999,
    \"desc\": \"$(python3 -c 'print("a" * 501)')\"
  }" | jq .
echo -e "\n"

# Test 8: Invalid JSON format
echo "Test 8: Invalid JSON format (should fail)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{"blacklist_type": "ClientId", invalid json}' | jq .
echo -e "\n"

# Test 9: Missing required fields
echo "Test 9: Missing required fields (should fail)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "ClientId"
  }' | jq .
echo -e "\n"

# Test 10: All valid blacklist types
echo "Test 10: Valid blacklist_type=IpAddress (should succeed)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "IpAddress",
    "resource_name": "192.168.1.100",
    "end_time": 9999999999,
    "desc": "Test IP blacklist"
  }' | jq .
echo -e "\n"

echo "Test 11: Valid blacklist_type=Username (should succeed)"
curl -X POST "${BASE_URL}/mqtt/blacklist/create" \
  -H "Content-Type: application/json" \
  -d '{
    "blacklist_type": "Username",
    "resource_name": "test_user",
    "end_time": 9999999999,
    "desc": "Test user blacklist"
  }' | jq .
echo -e "\n"

echo "======================================"
echo "Tests completed!"
echo "======================================"

