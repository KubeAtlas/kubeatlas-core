#!/bin/bash

echo "=== Testing User Deletion Functionality ==="

# Get admin user token
echo "1. Getting admin user token..."
ADMIN_TOKEN=$(curl -s -X POST http://localhost:8081/realms/kubeatlas/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=kubeatlas-backend" \
  -d "client_secret=backend-secret-key" \
  -d "username=admin-service" \
  -d "password=AdminPassw0rd!" | jq -r '.access_token')

if [ "$ADMIN_TOKEN" = "null" ] || [ -z "$ADMIN_TOKEN" ]; then
    echo "❌ Failed to get admin token"
    exit 1
fi

echo "✅ Admin token obtained: ${ADMIN_TOKEN:0:50}..."

# Test token validation
echo -e "\n2. Testing token validation..."
VALIDATION_RESULT=$(curl -s -X POST http://localhost:3001/auth/validate \
  -H "Content-Type: application/json" \
  -d "{\"token\": \"$ADMIN_TOKEN\"}")

echo "Token validation result: $VALIDATION_RESULT"

# Create a test user for deletion
echo -e "\n3. Creating test user for deletion..."
CREATE_RESPONSE=$(curl -s -X POST http://localhost:3001/api/v1/admin/users \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser_delete_demo",
    "email": "testuser_delete_demo@example.com",
    "first_name": "Test",
    "last_name": "Delete",
    "password": "TestPassword123!",
    "roles": ["user"]
  }')

echo "Create user response: $CREATE_RESPONSE"

# Extract user ID
USER_ID=$(echo $CREATE_RESPONSE | jq -r '.id')
echo "Created user ID: $USER_ID"

if [ "$USER_ID" = "null" ] || [ -z "$USER_ID" ]; then
    echo "❌ Failed to create test user"
    exit 1
fi

# Delete the user
echo -e "\n4. Testing DELETE endpoint..."
DELETE_RESPONSE=$(curl -s -X DELETE "http://localhost:3001/api/v1/admin/users/$USER_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

echo "Delete user response: $DELETE_RESPONSE"

# Verify deletion
echo -e "\n5. Verifying user deletion..."
VERIFY_RESPONSE=$(curl -s -w "%{http_code}" -X DELETE "http://localhost:3001/api/v1/admin/users/$USER_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

echo "Verification response (should be 400 or similar): $VERIFY_RESPONSE"

echo -e "\n=== Test completed ==="