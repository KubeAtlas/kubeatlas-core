
-- Подключенные контроллеры и агенты (для mTLS соединений)
CREATE TABLE IF NOT EXISTS connected_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    service_type VARCHAR(50) NOT NULL, -- 'controller' или 'agent'
    service_name VARCHAR(255) NOT NULL,
    controller_name VARCHAR(255), -- имя контроллера для агентов
    client_cert_serial VARCHAR(255) UNIQUE NOT NULL,
    client_cert_fingerprint VARCHAR(255) NOT NULL,
    connected_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    status VARCHAR(50) NOT NULL DEFAULT 'active'
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_connected_services_type ON connected_services(service_type);
CREATE INDEX IF NOT EXISTS idx_connected_services_cert_serial ON connected_services(client_cert_serial);
CREATE INDEX IF NOT EXISTS idx_connected_services_controller ON connected_services(controller_name);