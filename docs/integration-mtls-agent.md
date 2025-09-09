/# mTLS Agent Integration Guide

This document provides complete instructions for implementing and installing an mTLS agent that can successfully connect to the KubeAtlas Backend service.

## Overview

The KubeAtlas Backend uses a secure agent registration flow with one-time install tokens and mTLS authentication:

1. **Install Token Generation**: Backend generates one-time tokens via authenticated API
2. **Agent Registration**: Agent uses install token to register with backend
3. **mTLS Authentication**: Subsequent connections use client certificates for authentication

## Prerequisites

- Access to KubeAtlas Backend API
- Valid JWT token for install token generation (admin or authorized user)
- PKI infrastructure for certificate generation
- Network connectivity to backend service

## Architecture Overview

```
[Admin/UI] → [Backend API] → [Redis Storage]
     ↓
[Install Token] → [Agent] → [PKI Service] → [mTLS Connection]
```

## Step-by-Step Integration

### 1. Install Token Generation

First, obtain an install token from the backend API:

```bash
# Get JWT token (replace with your authentication method)
JWT_TOKEN=$(curl -s -X POST 'http://localhost:8081/realms/kubeatlas/protocol/openid-connect/token' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'grant_type=password' \
  -d 'client_id=kubeatlas-backend' \
  -d 'client_secret=backend-secret-key' \
  -d 'username=your-username' \
  -d 'password=your-password' | jq -r .access_token)

# Generate install token
INSTALL_TOKEN=$(curl -s -X POST http://localhost:3001/api/v1/install-tokens \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "service_name": "my-agent-name",
    "service_type": "agent",
    "controller_name": "my-controller"
  }' | jq -r .install_token)

echo "Install Token: $INSTALL_TOKEN"
```

### 2. Certificate Generation

Generate client certificates using your PKI service. The agent must have:

**Required Certificate Components:**
- Client certificate (PEM format)
- Private key (PEM format) 
- CA certificate (for chain validation)

**Certificate Requirements:**
- Valid X.509 client certificate
- Must be signed by trusted CA
- Subject should include agent identification
- Extended Key Usage: Client Authentication

**Example OpenSSL Certificate Generation:**
```bash
# Generate private key
openssl genrsa -out agent.key 2048

# Generate certificate signing request
openssl req -new -key agent.key -out agent.csr \
  -subj "/C=US/O=KubeAtlas/CN=agent-${AGENT_NAME}"

# Sign certificate (replace with your CA)
openssl x509 -req -in agent.csr -CA ca.crt -CAkey ca.key \
  -CAcreateserial -out agent.crt -days 365 \
  -extensions v3_req -extfile <(cat <<EOF
[v3_req]
keyUsage = keyEncipherment, dataEncipherment
extendedKeyUsage = clientAuth
EOF
)
```

### 3. Agent Registration

Register the agent with the backend using the install token:

```bash
# Prepare certificate content (ensure proper escaping)
CLIENT_CERT=$(cat agent.crt | sed ':a;N;$!ba;s/\n/\\n/g')

# Register agent
curl -X POST http://localhost:3001/api/v1/register \
  -H "Content-Type: application/json" \
  -d '{
    "install_token": "'$INSTALL_TOKEN'",
    "service_name": "my-agent-name",
    "service_type": "agent",
    "controller_name": "my-controller",
    "client_cert_pem": "'$CLIENT_CERT'",
    "metadata": {
      "version": "1.0.0",
      "environment": "production",
      "hostname": "'$(hostname)'",
      "platform": "'$(uname -s)'",
      "agent_id": "unique-agent-identifier"
    }
  }'
```

**Expected Response:**
```json
{
  "service_id": "uuid-of-registered-service",
  "message": "agent 'my-agent-name' successfully registered"
}
```

### 4. mTLS Connection Implementation

#### Connection Configuration

Configure your agent to use mTLS for subsequent API calls:

```bash
# Example using curl with client certificate
curl --cert agent.crt --key agent.key --cacert ca.crt \
  -H "Content-Type: application/json" \
  https://backend-service:3001/api/v1/agent/heartbeat \
  -d '{"agent_id": "unique-agent-identifier", "status": "healthy"}'
```

#### Programming Language Examples

**Go Example:**
```go
package main

import (
    "crypto/tls"
    "crypto/x509"
    "io/ioutil"
    "net/http"
)

func createMTLSClient(certFile, keyFile, caFile string) (*http.Client, error) {
    // Load client certificate
    cert, err := tls.LoadX509KeyPair(certFile, keyFile)
    if err != nil {
        return nil, err
    }

    // Load CA certificate
    caCert, err := ioutil.ReadFile(caFile)
    if err != nil {
        return nil, err
    }
    
    caCertPool := x509.NewCertPool()
    caCertPool.AppendCertsFromPEM(caCert)

    // Configure TLS
    tlsConfig := &tls.Config{
        Certificates: []tls.Certificate{cert},
        RootCAs:      caCertPool,
        ServerName:   "backend-service", // Must match server certificate
    }

    return &http.Client{
        Transport: &http.Transport{
            TLSClientConfig: tlsConfig,
        },
    }, nil
}
```

**Python Example:**
```python
import requests
import ssl

def create_mtls_session(cert_file, key_file, ca_file):
    session = requests.Session()
    session.cert = (cert_file, key_file)
    session.verify = ca_file
    
    return session

# Usage
session = create_mtls_session('agent.crt', 'agent.key', 'ca.crt')
response = session.post('https://backend-service:3001/api/v1/agent/heartbeat',
                       json={'agent_id': 'unique-agent-identifier', 'status': 'healthy'})
```

**Rust Example:**
```rust
use reqwest::Client;
use std::fs;

async fn create_mtls_client() -> Result<Client, Box<dyn std::error::Error>> {
    let cert = fs::read("agent.crt")?;
    let key = fs::read("agent.key")?;
    let ca = fs::read("ca.crt")?;

    let identity = reqwest::Identity::from_pem(&[cert, key].concat())?;
    let ca_cert = reqwest::Certificate::from_pem(&ca)?;

    let client = Client::builder()
        .identity(identity)
        .add_root_certificate(ca_cert)
        .build()?;

    Ok(client)
}
```

## Agent Implementation Requirements

### 1. Registration Flow

Your agent must implement the following registration sequence:

```
1. Receive install token (via command line, config file, or environment)
2. Generate or obtain client certificates from PKI service
3. Call registration API with install token and certificate
4. Store service_id for future reference
5. Begin normal operation with mTLS authentication
```

### 2. Certificate Management

**Certificate Storage:**
- Store certificates securely (proper file permissions: 600)
- Protect private keys from unauthorized access
- Consider using hardware security modules (HSM) for production

**Certificate Rotation:**
- Implement automatic certificate renewal before expiration
- Handle certificate rotation without service interruption
- Monitor certificate validity and alert on upcoming expiration

### 3. Error Handling

**Registration Errors:**
```json
// Invalid install token
{
  "error": "Failed to register service",
  "message": "Invalid or expired install token"
}

// Missing required fields
{
  "error": "Validation error",
  "message": "Missing required field: client_cert_pem"
}

// Certificate parsing error
{
  "error": "Failed to register service", 
  "message": "Invalid certificate format"
}
```

**Connection Errors:**
- Handle TLS handshake failures
- Implement exponential backoff for reconnection
- Log certificate validation errors appropriately

### 4. Health Monitoring

Implement periodic health checks and status reporting:

```bash
# Example heartbeat endpoint (to be implemented)
POST /api/v1/agent/heartbeat
{
  "agent_id": "unique-agent-identifier",
  "status": "healthy|degraded|unhealthy",
  "timestamp": "2025-09-08T18:00:00Z",
  "metrics": {
    "cpu_usage": 15.5,
    "memory_usage": 512,
    "disk_usage": 85.2
  }
}
```

## Security Best Practices

### Certificate Security
1. **Private Key Protection**: Store private keys with restricted permissions (600)
2. **Certificate Validation**: Always validate server certificates
3. **Revocation Checking**: Implement CRL/OCSP checking if required
4. **Key Rotation**: Plan for regular certificate rotation

### Network Security
1. **TLS Configuration**: Use TLS 1.2 or higher
2. **Cipher Suites**: Configure secure cipher suites only
3. **Certificate Pinning**: Consider pinning backend service certificates
4. **Network Isolation**: Restrict network access to backend service

### Operational Security
1. **Logging**: Log authentication events (avoid logging sensitive data)
2. **Monitoring**: Monitor failed authentication attempts
3. **Alerts**: Alert on certificate expiration and validation failures
4. **Backup**: Securely backup certificates and configuration

## Environment-Specific Configuration

### Development Environment
```bash
# Backend URL
BACKEND_URL=http://localhost:3001

# Certificate paths
CERT_PATH=/path/to/dev-certs/agent.crt
KEY_PATH=/path/to/dev-certs/agent.key
CA_PATH=/path/to/dev-certs/ca.crt

# Install token (for development only)
INSTALL_TOKEN=dev-token-12345
```

### Production Environment
```bash
# Backend URL (use proper domain/ingress)
BACKEND_URL=https://kubeatlas-backend.company.com

# Certificate paths (secure storage)
CERT_PATH=/etc/kubeatlas/certs/agent.crt
KEY_PATH=/etc/kubeatlas/certs/agent.key
CA_PATH=/etc/kubeatlas/certs/ca.crt

# Install token (from secure configuration management)
INSTALL_TOKEN_PATH=/etc/kubeatlas/secrets/install-token
```

## Troubleshooting

### Common Issues

**1. "Invalid or expired install token"**
- Verify install token was generated correctly
- Check token expiration (tokens have TTL)
- Ensure install token is used only once

**2. "Certificate validation failed"**
- Verify certificate format (PEM)
- Check certificate is not expired
- Ensure certificate chain is complete
- Validate certificate matches expected CN/SAN

**3. "TLS handshake failed"**
- Verify client certificate is properly formatted
- Check private key matches certificate
- Ensure CA certificate is trusted by backend
- Verify TLS version compatibility

**4. "Connection refused"**
- Check backend service is running
- Verify network connectivity
- Confirm firewall rules allow connection
- Check service discovery/DNS resolution

### Debug Commands

```bash
# Test certificate validity
openssl x509 -in agent.crt -text -noout

# Verify certificate chain
openssl verify -CAfile ca.crt agent.crt

# Test TLS connection
openssl s_client -connect backend-service:3001 \
  -cert agent.crt -key agent.key -CAfile ca.crt

# Check certificate fingerprint
openssl x509 -in agent.crt -fingerprint -sha256 -noout
```

## Integration Checklist

- [ ] Install token generated and stored securely
- [ ] Client certificates generated with proper attributes
- [ ] Agent registration completed successfully
- [ ] mTLS connection established and tested
- [ ] Certificate rotation mechanism implemented
- [ ] Error handling and logging implemented
- [ ] Health monitoring and heartbeat configured
- [ ] Security best practices applied
- [ ] Environment-specific configuration tested
- [ ] Documentation and operational procedures created

## Support and Contact

For technical support and integration assistance:
- Check backend logs: `docker compose logs backend`
- Review Redis data: `docker exec redis redis-cli KEYS "*"`
- Monitor service status: `curl http://localhost:3001/health`

## API Reference

### Registration Endpoint
```
POST /api/v1/register
Content-Type: application/json

{
  "install_token": "string (required)",
  "service_name": "string (required)",
  "service_type": "agent|controller (required)",
  "controller_name": "string (required for agents)",
  "client_cert_pem": "string (required)",
  "metadata": {
    "version": "string",
    "environment": "string",
    "hostname": "string",
    "platform": "string",
    "agent_id": "string"
  }
}
```

### Install Token Generation
```
POST /api/v1/install-tokens
Authorization: Bearer JWT_TOKEN
Content-Type: application/json

{
  "service_name": "string (required)",
  "service_type": "agent|controller (required)",
  "controller_name": "string (optional)"
}
```

This guide provides all necessary information for implementing a secure mTLS agent integration with the KubeAtlas Backend service.