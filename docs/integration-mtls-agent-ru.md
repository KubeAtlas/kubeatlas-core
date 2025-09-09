# Руководство по интеграции mTLS агента

Этот документ содержит полные инструкции по реализации и установке mTLS агента, который может успешно подключиться к сервису KubeAtlas Backend.

## Обзор

KubeAtlas Backend использует безопасный процесс регистрации агентов с одноразовыми токенами установки и аутентификацией mTLS:

1. **Генерация токена установки**: Backend генерирует одноразовые токены через аутентифицированный API
2. **Регистрация агента**: Агент использует токен установки для регистрации в backend
3. **Аутентификация mTLS**: Последующие подключения используют клиентские сертификаты для аутентификации

## Предварительные требования

- Доступ к API KubeAtlas Backend
- Действительный JWT токен для генерации токена установки (администратор или авторизованный пользователь)
- PKI инфраструктура для генерации сертификатов
- Сетевое подключение к сервису backend
- **PCI сервер** для регистрации агентов

## Обзор архитектуры

```
[Админ/UI] → [Backend API] → [Redis Storage]
     ↓
[Токен установки] → [Агент] → [PCI сервер] → [mTLS подключение]
```

## Пошаговая интеграция

### 1. Генерация токена установки

Сначала получите токен установки из backend API:

```bash
# Получите JWT токен (замените на ваш метод аутентификации)
JWT_TOKEN=$(curl -s -X POST 'http://localhost:8081/realms/kubeatlas/protocol/openid-connect/token' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  -d 'grant_type=password' \
  -d 'client_id=kubeatlas-backend' \
  -d 'client_secret=backend-secret-key' \
  -d 'username=your-username' \
  -d 'password=your-password' | jq -r .access_token)

# Сгенерируйте токен установки
INSTALL_TOKEN=$(curl -s -X POST http://localhost:3001/api/v1/install-tokens \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "service_name": "my-agent-name",
    "service_type": "agent",
    "controller_name": "my-controller"
  }' | jq -r .install_token)

echo "Токен установки: $INSTALL_TOKEN"
```

### 2. Генерация сертификатов

Сгенерируйте клиентские сертификаты используя ваш PCI сервер. Агент должен иметь:

**Обязательные компоненты сертификата:**
- Клиентский сертификат (формат PEM)
- Приватный ключ (формат PEM) 
- Сертификат CA (для проверки цепи)

**Требования к сертификату:**
- Действительный клиентский сертификат X.509
- Должен быть подписан доверенным CA
- Subject должен включать идентификацию агента
- Extended Key Usage: Client Authentication

**Пример генерации сертификата через PCI сервер:**
```bash
# Генерируйте приватный ключ
openssl genrsa -out agent.key 2048

# Генерируйте запрос на подпись сертификата
openssl req -new -key agent.key -out agent.csr \
  -subj "/C=RU/O=KubeAtlas/CN=agent-${AGENT_NAME}"

# Подпишите сертификат через PCI сервер (замените на ваш PCI сервер)
curl -X POST https://pci-server:8443/api/v1/sign-certificate \
  -H "Content-Type: application/json" \
  -d '{
    "csr": "'$(cat agent.csr | base64 -w 0)'",
    "agent_name": "'$AGENT_NAME'",
    "controller": "my-controller"
  }' | jq -r .certificate > agent.crt
```

### 3. Регистрация агента

Зарегистрируйте агента в backend используя токен установки:

```bash
# Подготовьте содержимое сертификата (убедитесь в правильном экранировании)
CLIENT_CERT=$(cat agent.crt | sed ':a;N;$!ba;s/\n/\\n/g')

# Зарегистрируйте агента
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
      "agent_id": "unique-agent-identifier",
      "pci_server": "https://pci-server:8443"
    }
  }'
```

**Ожидаемый ответ:**
```json
{
  "service_id": "uuid-зарегистрированного-сервиса",
  "message": "агент 'my-agent-name' успешно зарегистрирован"
}
```

### 4. Реализация mTLS подключения

#### Конфигурация подключения

Настройте ваш агент для использования mTLS для последующих API вызовов:

```bash
# Пример использования curl с клиентским сертификатом
curl --cert agent.crt --key agent.key --cacert ca.crt \
  -H "Content-Type: application/json" \
  https://backend-service:3001/api/v1/agent/heartbeat \
  -d '{"agent_id": "unique-agent-identifier", "status": "healthy"}'
```

#### Примеры для языков программирования

**Пример на Go:**
```go
package main

import (
    "crypto/tls"
    "crypto/x509"
    "io/ioutil"
    "net/http"
)

func createMTLSClient(certFile, keyFile, caFile string) (*http.Client, error) {
    // Загрузите клиентский сертификат
    cert, err := tls.LoadX509KeyPair(certFile, keyFile)
    if err != nil {
        return nil, err
    }

    // Загрузите CA сертификат
    caCert, err := ioutil.ReadFile(caFile)
    if err != nil {
        return nil, err
    }
    
    caCertPool := x509.NewCertPool()
    caCertPool.AppendCertsFromPEM(caCert)

    // Настройте TLS
    tlsConfig := &tls.Config{
        Certificates: []tls.Certificate{cert},
        RootCAs:      caCertPool,
        ServerName:   "backend-service", // Должно соответствовать серверному сертификату
    }

    return &http.Client{
        Transport: &http.Transport{
            TLSClientConfig: tlsConfig,
        },
    }, nil
}
```

**Пример на Python:**
```python
import requests
import ssl

def create_mtls_session(cert_file, key_file, ca_file):
    session = requests.Session()
    session.cert = (cert_file, key_file)
    session.verify = ca_file
    
    return session

# Использование
session = create_mtls_session('agent.crt', 'agent.key', 'ca.crt')
response = session.post('https://backend-service:3001/api/v1/agent/heartbeat',
                       json={'agent_id': 'unique-agent-identifier', 'status': 'healthy'})
```

**Пример на Rust:**
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

## Требования к реализации агента

### 1. Процесс регистрации

Ваш агент должен реализовывать следующую последовательность регистрации:

```
1. Получить токен установки (через командную строку, конфигурационный файл или переменные среды)
2. Сгенерировать или получить клиентские сертификаты от PCI сервера
3. Вызвать API регистрации с токеном установки и сертификатом
4. Сохранить service_id для будущих ссылок
5. Начать нормальную работу с аутентификацией mTLS
```

### 2. Управление сертификатами

**Хранение сертификатов:**
- Храните сертификаты безопасно (правильные права доступа к файлам: 600)
- Защитите приватные ключи от несанкционированного доступа
- Рассмотрите использование модулей аппаратной безопасности (HSM) для продакшена

**Ротация сертификатов:**
- Реализуйте автоматическое обновление сертификатов до истечения срока
- Обрабатывайте ротацию сертификатов без прерывания сервиса
- Отслеживайте действительность сертификатов и предупреждайте о приближающемся истечении срока
- Интегрируйтесь с PCI сервером для автоматической ротации

### 3. Обработка ошибок

**Ошибки регистрации:**
```json
// Недействительный токен установки
{
  "error": "Не удалось зарегистрировать сервис",
  "message": "Недействительный или просроченный токен установки"
}

// Отсутствуют обязательные поля
{
  "error": "Ошибка валидации",
  "message": "Отсутствует обязательное поле: client_cert_pem"
}

// Ошибка парсинга сертификата
{
  "error": "Не удалось зарегистрировать сервис", 
  "message": "Неверный формат сертификата"
}
```

**Ошибки подключения:**
- Обрабатывайте сбои TLS handshake
- Реализуйте экспоненциальную задержку для переподключения
- Правильно логируйте ошибки валидации сертификатов

### 4. Мониторинг здоровья

Реализуйте периодические проверки здоровья и отчетность о статусе:

```bash
# Пример endpoint для heartbeat (должен быть реализован)
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

## Лучшие практики безопасности

### Безопасность сертификатов
1. **Защита приватного ключа**: Храните приватные ключи с ограниченными правами (600)
2. **Валидация сертификатов**: Всегда валидируйте серверные сертификаты
3. **Проверка отзыва**: Реализуйте проверку CRL/OCSP при необходимости
4. **Ротация ключей**: Планируйте регулярную ротацию сертификатов через PCI сервер

### Сетевая безопасность
1. **Конфигурация TLS**: Используйте TLS 1.2 или выше
2. **Шифровальные наборы**: Настройте только безопасные шифровальные наборы
3. **Привязка сертификатов**: Рассмотрите привязку сертификатов сервиса backend
4. **Изоляция сети**: Ограничьте сетевой доступ к сервису backend

### Операционная безопасность
1. **Логирование**: Логируйте события аутентификации (избегайте логирования чувствительных данных)
2. **Мониторинг**: Отслеживайте неудачные попытки аутентификации
3. **Предупреждения**: Предупреждайте об истечении сертификатов и ошибках валидации
4. **Резервное копирование**: Безопасно создавайте резервные копии сертификатов и конфигурации

## Конфигурация для конкретных сред

### Среда разработки
```bash
# URL Backend'а
BACKEND_URL=http://localhost:3001

# Пути к сертификатам
CERT_PATH=/path/to/dev-certs/agent.crt
KEY_PATH=/path/to/dev-certs/agent.key
CA_PATH=/path/to/dev-certs/ca.crt

# PCI сервер (для разработки)
PCI_SERVER_URL=https://dev-pci-server:8443

# Токен установки (только для разработки)
INSTALL_TOKEN=dev-token-12345
```

### Продуктовая среда
```bash
# URL Backend'а (используйте правильный домен/ingress)
BACKEND_URL=https://kubeatlas-backend.company.com

# Пути к сертификатам (безопасное хранение)
CERT_PATH=/etc/kubeatlas/certs/agent.crt
KEY_PATH=/etc/kubeatlas/certs/agent.key
CA_PATH=/etc/kubeatlas/certs/ca.crt

# PCI сервер (продуктовый)
PCI_SERVER_URL=https://pci-server.company.com:8443

# Токен установки (из безопасного управления конфигурацией)
INSTALL_TOKEN_PATH=/etc/kubeatlas/secrets/install-token
```

## Устранение неисправностей

### Частые проблемы

**1. "Недействительный или просроченный токен установки"**
- Проверьте, что токен установки был сгенерирован правильно
- Проверьте истечение срока токена (токены имеют TTL)
- Убедитесь, что токен установки используется только один раз

**2. "Сбой валидации сертификата"**
- Проверьте формат сертификата (PEM)
- Убедитесь, что сертификат не просрочен
- Убедитесь, что цепь сертификатов полная
- Проверьте, что сертификат соответствует ожидаемому CN/SAN

**3. "Сбой TLS handshake"**
- Проверьте, что клиентский сертификат правильно отформатирован
- Убедитесь, что приватный ключ соответствует сертификату
- Убедитесь, что сертификат CA доверенный для backend'а
- Проверьте совместимость версий TLS

**4. "Соединение отклонено"**
- Проверьте, что сервис backend работает
- Проверьте сетевое подключение
- Подтвердите, что правила firewall разрешают соединение
- Проверьте service discovery/DNS разрешение

### Отладочные команды

```bash
# Проверьте действительность сертификата
openssl x509 -in agent.crt -text -noout

# Проверьте цепь сертификатов
openssl verify -CAfile ca.crt agent.crt

# Протестируйте TLS соединение
openssl s_client -connect backend-service:3001 \
  -cert agent.crt -key agent.key -CAfile ca.crt

# Проверьте отпечаток сертификата
openssl x509 -in agent.crt -fingerprint -sha256 -noout

# Проверьте подключение к PCI серверу
curl -k https://pci-server:8443/health
```

## Чек-лист интеграции

- [ ] Токен установки сгенерирован и безопасно сохранен
- [ ] Клиентские сертификаты сгенерированы через PCI сервер с правильными атрибутами
- [ ] Регистрация агента завершена успешно
- [ ] mTLS соединение установлено и протестировано
- [ ] Механизм ротации сертификатов через PCI сервер реализован
- [ ] Обработка ошибок и логирование реализованы
- [ ] Мониторинг здоровья и heartbeat настроены
- [ ] Применены лучшие практики безопасности
- [ ] Конфигурация для конкретной среды протестирована
- [ ] Создана документация и операционные процедуры

## Поддержка и контакты

Для технической поддержки и помощи с интеграцией:
- Проверьте логи backend: `docker compose logs backend`
- Просмотрите данные Redis: `docker exec redis redis-cli KEYS "*"`
- Отслеживайте статус сервиса: `curl http://localhost:3001/health`
- Проверьте статус PCI сервера: `curl -k https://pci-server:8443/health`

## Справочник API

### Endpoint регистрации
```
POST /api/v1/register
Content-Type: application/json

{
  "install_token": "string (обязательно)",
  "service_name": "string (обязательно)",
  "service_type": "agent|controller (обязательно)",
  "controller_name": "string (обязательно для агентов)",
  "client_cert_pem": "string (обязательно)",
  "metadata": {
    "version": "string",
    "environment": "string",
    "hostname": "string",
    "platform": "string",
    "agent_id": "string",
    "pci_server": "string (URL PCI сервера)"
  }
}
```

### Генерация токена установки
```
POST /api/v1/install-tokens
Authorization: Bearer JWT_TOKEN
Content-Type: application/json

{
  "service_name": "string (обязательно)",
  "service_type": "agent|controller (обязательно)",
  "controller_name": "string (опционально)"
}
```

### PCI сервер API (примерная спецификация)
```
POST /api/v1/sign-certificate
Content-Type: application/json

{
  "csr": "string (base64 encoded CSR)",
  "agent_name": "string",
  "controller": "string",
  "validity_days": 365
}
```

Это руководство предоставляет всю необходимую информацию для реализации безопасной интеграции mTLS агента с сервисом KubeAtlas Backend через PCI сервер.