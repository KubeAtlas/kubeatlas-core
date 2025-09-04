# Интеграция фронтенда с KubeAtlas Backend + Keycloak

## 1) Конфигурация Keycloak на фронтенде

- Realm: `kubeatlas`
- Auth URL: `http://localhost:8081/realms/kubeatlas`
- Client ID: `kubeatlas-backend`
- Grant flow: PKCE (рекомендуется) или password (только для сервер-сайда)

Для SPA используйте официальную библиотеку keycloak-js:
```
npm i keycloak-js
```
Пример инициализации:
```js
import Keycloak from 'keycloak-js';

export const keycloak = new Keycloak({
  url: 'http://localhost:8081',
  realm: 'kubeatlas',
  clientId: 'kubeatlas-backend'
});

export async function initAuth() {
  const authenticated = await keycloak.init({
    onLoad: 'login-required',
    checkLoginIframe: false,
    pkceMethod: 'S256'
  });
  return authenticated;
}
```

## 2) Передача токена в бэкенд

Во все запросы к бэкенду добавляйте заголовок:
```
Authorization: Bearer <access_token>
```
Пример (fetch):
```js
const res = await fetch('http://localhost:3001/api/v1/user/roles', {
  headers: {
    Authorization: `Bearer ${keycloak.token}`
  }
});
const data = await res.json();
```

## 3) Пример вызовов

### Пользовательские эндпоинты (требуют аутентификации)

- Роли пользователя:
```
GET http://localhost:3001/api/v1/user/roles
```
- Профиль:
```
GET http://localhost:3001/api/v1/user/profile
```

### Административные эндпоинты (требуют роль admin)

- Создание пользователя:
```
POST http://localhost:3001/api/v1/admin/users
Body: { username, email, first_name, last_name, password, roles: ["user"] }
```

- Обновление пользователя:
```
PUT http://localhost:3001/api/v1/admin/users/{user_id}
Body: { first_name?, last_name?, email?, roles? }
```

- Удаление пользователя:
```
DELETE http://localhost:3001/api/v1/admin/users/{user_id}
```

- Получение активных сессий пользователя:
```
GET http://localhost:3001/api/v1/admin/users/{user_id}/sessions
```

- Отзыв всех сессий пользователя:
```
POST http://localhost:3001/api/v1/admin/users/{user_id}/sessions/revoke
```

## 4) Обновление токена

Перед каждым запросом обновляйте токен (если используете keycloak-js):
```js
await keycloak.updateToken(30).catch(() => keycloak.login());
```

## 5) CORS

Бэкенд уже включает CORS (allow origin: Any). Для продакшена задайте разрешённые домены.
