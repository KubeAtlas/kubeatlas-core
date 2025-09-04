# API KubeAtlas Backend

Base URL: `http://localhost:3001`

Все защищённые эндпоинты требуют заголовок:
```
Authorization: Bearer <access_token>
```

## Health
- GET `/health`

## Auth
- POST `/auth/validate` — проверить токен
- GET `/auth/user` — получить сведения о пользователе

## User (защищено)
- GET `/api/v1/user/profile` — профиль текущего пользователя
- GET `/api/v1/user/roles` — список ролей, флаги isAdmin/isUser/isGuest

## Admin (требует роль `admin`)

### Управление пользователями

- POST `/api/v1/admin/users` — создание пользователя
```json
{
  "username": "john",
  "email": "john@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "password": "StrongPassw0rd!",
  "roles": ["user"]
}
```

- PUT `/api/v1/admin/users/:id` — обновление пользователя
```json
{
  "email": "new@example.com",
  "first_name": "New",
  "last_name": "Name",
  "roles": ["user"]
}
```

- DELETE `/api/v1/admin/users/:id` — удаление пользователя
  - ⚠️ **Осторожно**: необратимое действие!
  - Ответ: `{"message": "User deleted successfully", "id": "user-uuid"}`

### Управление сессиями

- GET `/api/v1/admin/users/:id/sessions` — получить активные сессии пользователя
- POST `/api/v1/admin/users/:id/sessions/revoke` — отозвать все сессии пользователя

### Пример использования

```bash
# Получение токена администратора
TOKEN=$(curl -s -X POST 'http://localhost:8081/realms/kubeatlas/protocol/openid-connect/token' \
  -d grant_type=password -d client_id=kubeatlas-backend -d client_secret=backend-secret-key \
  -d username=admin-service -d password='AdminPassw0rd!' | jq -r '.access_token')

# Создание пользователя
curl -X POST http://localhost:3001/api/v1/admin/users \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","first_name":"Test","last_name":"User","password":"TestPass123!","roles":["user"]}'

# Удаление пользователя
curl -X DELETE http://localhost:3001/api/v1/admin/users/USER_ID \
  -H "Authorization: Bearer $TOKEN"
```
