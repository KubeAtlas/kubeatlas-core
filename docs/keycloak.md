# Настройка Keycloak для KubeAtlas Backend

## Быстрый старт через docker-compose

- Поднимаем:
```
docker compose up -d --build
```
- Keycloak доступен на `http://localhost:8081`.
- Импортируется realm `kubeatlas` и клиент `kubeatlas-backend`.

## Важное
- Сервис ждёт готовности Keycloak при старте.
- `ADM_USER`/`ADM_PASSWORD` — создаёт/обеспечивает пользователя и роль admin.
- `KEYCLOAK_ADMIN_USER`/`KEYCLOAK_ADMIN_PASSWORD` — присваивает роль admin через Admin API.

## Проверка токена
```
# Получить токен
curl -s -X POST 'http://localhost:8081/realms/kubeatlas/protocol/openid-connect/token' \
  -d grant_type=password -d client_id=kubeatlas-backend -d client_secret=backend-secret-key \
  -d username=admin-service -d password='AdminPassw0rd!' -d 'scope=openid profile email roles'
```

## JWKS
- Бэкенд валидирует JWT локально по JWKS: `/realms/kubeatlas/protocol/openid-connect/certs`.

## Роли
- Realm-роль `admin`, пользователь `admin-service` получает её автоматически на старте.
