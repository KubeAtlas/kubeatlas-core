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
- POST `/api/v1/admin/users`
```
{
  "username": "john",
  "email": "john@example.com",
  "first_name": "John",
  "last_name": "Doe",
  "password": "StrongPassw0rd!",
  "roles": ["user"]
}
```
- PUT `/api/v1/admin/users/:id`
```
{
  "email": "new@example.com",
  "first_name": "New",
  "last_name": "Name",
  "roles": ["user"]
}
```
