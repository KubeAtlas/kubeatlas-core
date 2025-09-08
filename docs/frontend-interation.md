# Интеграция фронтенда с KubeAtlas Backend + Keycloak

## Содержание

1. [Конфигурация Keycloak на фронтенде](#1-конфигурация-keycloak-на-фронтенде)
2. [Передача токена в бэкенд](#2-передача-токена-в-бэкенд)
3. [Примеры вызовов API](#3-пример-вызовов)
   - [Пользовательские эндпоинты](#пользовательские-эндпоинты-требуют-аутентификации)
   - [Эндпоинт статистики (новый)](#статистика-системы-новый-эндпоинт)
   - [Административные эндпоинты](#административные-эндпоинты-требуют-роль-admin)
4. [Обработка токенов и автоматическое обновление](#4-обработка-токенов-и-автоматическое-обновление)
5. [Обработка ошибок и лучшие практики](#5-обработка-ошибок-и-лучшие-практики)
6. [TypeScript типы](#6-typescript-типы-для-статистики)
7. [CORS](#7-cors)

---

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

- **Статистика системы** (новый эндпоинт):
```
GET http://localhost:3001/api/v1/statistics
```

Пример ответа статистики:
```json
{
  "success": true,
  "data": {
    "total_users": {
      "value": 1234,
      "change_percent": 12.0,
      "change_period": "с прошлого месяца"
    },
    "active_sessions": {
      "value": 89,
      "change_percent": 5.0,
      "change_period": "с прошлого часа"
    },
    "system_status": {
      "percentage": 98.5,
      "status": "Все системы работают",
      "details": [
        {
          "name": "Keycloak",
          "status": "operational",
          "uptime_percentage": 99.9
        },
        {
          "name": "Database",
          "status": "operational",
          "uptime_percentage": 99.5
        }
      ]
    }
  },
  "error": null,
  "message": null
}
```

Пример использования на фронтенде:
```js
// Получение статистики для дашборда
const getStatistics = async () => {
  try {
    await keycloak.updateToken(30);
    const response = await fetch('http://localhost:3001/api/v1/statistics', {
      headers: {
        'Authorization': `Bearer ${keycloak.token}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const data = await response.json();
    
    if (data.success) {
      // Обновляем карточки статистики в UI
      updateDashboardCards(data.data);
    } else {
      console.error('API error:', data.error);
    }
  } catch (error) {
    console.error('Failed to fetch statistics:', error);
    // Обработка ошибки в UI
  }
};

// Функция для обновления карточек дашборда
const updateDashboardCards = (stats) => {
  // Общее количество пользователей
  document.getElementById('total-users-count').textContent = stats.total_users.value;
  document.getElementById('total-users-change').textContent = 
    `+${stats.total_users.change_percent}% ${stats.total_users.change_period}`;
  
  // Активные сессии
  document.getElementById('active-sessions-count').textContent = stats.active_sessions.value;
  document.getElementById('active-sessions-change').textContent = 
    `+${stats.active_sessions.change_percent}% ${stats.active_sessions.change_period}`;
  
  // Статус системы
  document.getElementById('system-status-percentage').textContent = 
    `${stats.system_status.percentage}%`;
  document.getElementById('system-status-text').textContent = stats.system_status.status;
  
  // Обновление статуса сервисов
  stats.system_status.details.forEach(service => {
    const serviceElement = document.getElementById(`service-${service.name.toLowerCase()}`);
    if (serviceElement) {
      serviceElement.textContent = `${service.name}: ${service.status} (${service.uptime_percentage}%)`;
      serviceElement.className = service.status === 'operational' ? 'status-ok' : 'status-error';
    }
  });
};

// Автоматическое обновление статистики каждые 30 секунд
setInterval(getStatistics, 30000);
```

### Административные эндпоинты (требуют роль admin)

#### Управление пользователями

- **Получение всех пользователей:**
```
GET http://localhost:3001/api/v1/admin/users
```
Возвращает список всех пользователей в системе.

- **Получение пользователя по ID:**
```
GET http://localhost:3001/api/v1/admin/users/{user_id}
```
Возвращает информацию о конкретном пользователе.

- **Получение ролей пользователя:**
```
GET http://localhost:3001/api/v1/admin/users/{user_id}/roles
```
Возвращает список ролей пользователя.

- **Создание пользователя:**
```
POST http://localhost:3001/api/v1/admin/users
Body: { username, email, first_name, last_name, password, roles: ["user"] }
```

- **Обновление пользователя:**
```
PUT http://localhost:3001/api/v1/admin/users/{user_id}
Body: { first_name?, last_name?, email?, roles? }
```

- **Удаление пользователя:**
```
DELETE http://localhost:3001/api/v1/admin/users/{user_id}
```

#### Управление сессиями

- **Получение активных сессий пользователя:**
```
GET http://localhost:3001/api/v1/admin/users/{user_id}/sessions
```
Возвращает список всех активных сессий пользователя.

- **Отзыв всех сессий пользователя:**
```
POST http://localhost:3001/api/v1/admin/users/{user_id}/sessions/revoke
```
Закрывает все активные сессии пользователя.

- **Закрытие отдельной сессии (НОВОЕ):**
```
DELETE http://localhost:3001/api/v1/admin/users/{user_id}/sessions/{session_id}
```
Закрывает конкретную сессию пользователя по ID сессии.

#### Примеры использования новых эндпоинтов

**Получение списка пользователей для админ-панели:**
```js
const getAllUsers = async () => {
  try {
    await keycloak.updateToken(30);
    const response = await fetch('http://localhost:3001/api/v1/admin/users', {
      headers: {
        'Authorization': `Bearer ${keycloak.token}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const users = await response.json();
    return users;
  } catch (error) {
    console.error('Failed to fetch users:', error);
    throw error;
  }
};
```

**Получение информации о пользователе с его ролями:**
```js
const getUserDetails = async (userId) => {
  try {
    await keycloak.updateToken(30);
    
    // Параллельно получаем информацию о пользователе и его роли
    const [userResponse, rolesResponse] = await Promise.all([
      fetch(`http://localhost:3001/api/v1/admin/users/${userId}`, {
        headers: {
          'Authorization': `Bearer ${keycloak.token}`,
          'Content-Type': 'application/json'
        }
      }),
      fetch(`http://localhost:3001/api/v1/admin/users/${userId}/roles`, {
        headers: {
          'Authorization': `Bearer ${keycloak.token}`,
          'Content-Type': 'application/json'
        }
      })
    ]);
    
    if (!userResponse.ok || !rolesResponse.ok) {
      throw new Error('Failed to fetch user details');
    }
    
    const user = await userResponse.json();
    const roles = await rolesResponse.json();
    
    return { ...user, roles };
  } catch (error) {
    console.error('Failed to fetch user details:', error);
    throw error;
  }
};
```

**Управление сессиями пользователя:**
```js
const getUserSessions = async (userId) => {
  try {
    await keycloak.updateToken(30);
    const response = await fetch(`http://localhost:3001/api/v1/admin/users/${userId}/sessions`, {
      headers: {
        'Authorization': `Bearer ${keycloak.token}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const sessions = await response.json();
    return sessions;
  } catch (error) {
    console.error('Failed to fetch user sessions:', error);
    throw error;
  }
};

// Закрытие конкретной сессии
const revokeSpecificSession = async (userId, sessionId) => {
  try {
    await keycloak.updateToken(30);
    const response = await fetch(`http://localhost:3001/api/v1/admin/users/${userId}/sessions/${sessionId}`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${keycloak.token}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const result = await response.json();
    return result;
  } catch (error) {
    console.error('Failed to revoke session:', error);
    throw error;
  }
};

// Закрытие всех сессий пользователя
const revokeAllUserSessions = async (userId) => {
  try {
    await keycloak.updateToken(30);
    const response = await fetch(`http://localhost:3001/api/v1/admin/users/${userId}/sessions/revoke`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${keycloak.token}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const result = await response.json();
    return result;
  } catch (error) {
    console.error('Failed to revoke all sessions:', error);
    throw error;
  }
};
```

**Компонент управления сессиями для React:**
```jsx
import React, { useState, useEffect } from 'react';

const UserSessionsManager = ({ userId }) => {
  const [sessions, setSessions] = useState([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(null);

  useEffect(() => {
    loadUserSessions();
  }, [userId]);

  const loadUserSessions = async () => {
    try {
      setLoading(true);
      setError(null);
      const userSessions = await getUserSessions(userId);
      setSessions(userSessions);
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleRevokeSession = async (sessionId) => {
    try {
      await revokeSpecificSession(userId, sessionId);
      // Обновляем список сессий после закрытия
      await loadUserSessions();
    } catch (err) {
      setError(err.message);
    }
  };

  const handleRevokeAllSessions = async () => {
    try {
      await revokeAllUserSessions(userId);
      // Обновляем список сессий
      await loadUserSessions();
    } catch (err) {
      setError(err.message);
    }
  };

  if (loading) return <div>Загрузка сессий...</div>;
  if (error) return <div>Ошибка: {error}</div>;

  return (
    <div className="sessions-manager">
      <div className="sessions-header">
        <h3>Активные сессии ({sessions.length})</h3>
        <button 
          onClick={handleRevokeAllSessions}
          className="btn btn-danger"
          disabled={sessions.length === 0}
        >
          Закрыть все сессии
        </button>
      </div>
      
      {sessions.length === 0 ? (
        <p>Нет активных сессий</p>
      ) : (
        <div className="sessions-list">
          {sessions.map((session) => (
            <div key={session.id} className="session-card">
              <div className="session-info">
                <p><strong>ID:</strong> {session.id}</p>
                <p><strong>IP:</strong> {session.ipAddress}</p>
                <p><strong>Клиент:</strong> {session.clientId}</p>
                <p><strong>Начало:</strong> {new Date(session.start).toLocaleString()}</p>
              </div>
              <button
                onClick={() => handleRevokeSession(session.id)}
                className="btn btn-outline-danger btn-sm"
              >
                Закрыть
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default UserSessionsManager;
```

## 4) Обработка токенов и автоматическое обновление

### ⚠️ **КРИТИЧЕСКИ ВАЖНО: Обработка истекших токенов**

Токены в Keycloak имеют короткое время жизни (по умолчанию 1 час). Ваш фронтенд **ОБЯЗАТЕЛЬНО** должен:

1. **Проверять истечение токена** перед каждым запросом
2. **Автоматически обновлять токен** при необходимости
3. **Повторять запрос** с новым токеном при получении 401 ошибки

### Базовая обработка с keycloak-js:

```js
// Перед каждым запросом обновляйте токен (если используете keycloak-js)
await keycloak.updateToken(30).catch(() => keycloak.login());
```

### 🚀 **Продвинутая обработка токенов - рекомендуемый подход:**

```js
// utils/tokenManager.js
class TokenManager {
  constructor(keycloak) {
    this.keycloak = keycloak;
  }

  // Проверка и обновление токена с retry логикой
  async ensureValidToken(minValiditySeconds = 30) {
    try {
      // Попытка обновить токен
      const refreshed = await this.keycloak.updateToken(minValiditySeconds);
      if (refreshed) {
        console.log('Token was successfully refreshed');
      }
      return this.keycloak.token;
    } catch (error) {
      console.error('Failed to refresh token:', error);
      // Если не удалось обновить токен - перенаправляем на логин
      this.keycloak.login();
      throw new Error('Authentication required');
    }
  }

  // Выполнение API запроса с автоматическим обновлением токена
  async makeAuthenticatedRequest(url, options = {}) {
    const maxRetries = 2;
    let retryCount = 0;

    while (retryCount < maxRetries) {
      try {
        // Получаем валидный токен
        const token = await this.ensureValidToken();
        
        // Выполняем запрос
        const response = await fetch(url, {
          ...options,
          headers: {
            'Authorization': `Bearer ${token}`,
            'Content-Type': 'application/json',
            ...options.headers
          }
        });

        // Если получили 401 - токен протух во время запроса
        if (response.status === 401) {
          console.warn('Token expired during request, attempting refresh...');
          retryCount++;
          
          // Принудительно обновляем токен
          await this.keycloak.updateToken(-1); // Принудительное обновление
          continue; // Повторяем запрос
        }

        // Если другие ошибки HTTP - возвращаем как есть
        return response;

      } catch (error) {
        retryCount++;
        console.error(`Request attempt ${retryCount} failed:`, error);
        
        if (retryCount >= maxRetries) {
          throw error;
        }
        
        // Небольшая задержка перед повтором
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
  }

  // Получение нового токена через refresh token
  async refreshToken() {
    try {
      const refreshed = await this.keycloak.updateToken(-1); // Принудительное обновление
      if (refreshed) {
        console.log('Token refreshed successfully');
        return this.keycloak.token;
      }
      return this.keycloak.token;
    } catch (error) {
      console.error('Token refresh failed:', error);
      // Перенаправляем на логин если refresh не удался
      this.keycloak.login();
      throw error;
    }
  }

  // Проверка валидности токена
  isTokenValid(minValiditySeconds = 30) {
    return this.keycloak.isTokenExpired() === false && 
           this.keycloak.tokenParsed.exp > (Date.now() / 1000) + minValiditySeconds;
  }

  // Получение времени до истечения токена
  getTokenTimeLeft() {
    if (!this.keycloak.tokenParsed) return 0;
    const now = Math.ceil(Date.now() / 1000);
    return this.keycloak.tokenParsed.exp - now;
  }

  // Автоматическое обновление токена по таймеру
  startAutoRefresh() {
    const refreshInterval = setInterval(async () => {
      if (this.getTokenTimeLeft() < 300) { // Обновляем за 5 минут до истечения
        try {
          await this.refreshToken();
        } catch (error) {
          console.error('Auto refresh failed:', error);
          clearInterval(refreshInterval);
        }
      }
    }, 60000); // Проверяем каждую минуту

    return refreshInterval;
  }
}

// Создаем глобальный экземпляр
export const tokenManager = new TokenManager(keycloak);
```

### 📝 **Без keycloak-js (ручное управление токенами):**

Если вы не используете keycloak-js, вот пример ручного управления:

```js
// utils/authManager.js
class AuthManager {
  constructor() {
    this.accessToken = null;
    this.refreshToken = null;
    this.tokenExpiry = null;
  }

  // Сохранение токенов после логина
  setTokens(accessToken, refreshToken, expiresIn) {
    this.accessToken = accessToken;
    this.refreshToken = refreshToken;
    this.tokenExpiry = Date.now() + (expiresIn * 1000);
    
    // Сохраняем в localStorage для персистентности
    localStorage.setItem('access_token', accessToken);
    localStorage.setItem('refresh_token', refreshToken);
    localStorage.setItem('token_expiry', this.tokenExpiry.toString());
  }

  // Загрузка токенов из localStorage
  loadTokens() {
    this.accessToken = localStorage.getItem('access_token');
    this.refreshToken = localStorage.getItem('refresh_token');
    const expiry = localStorage.getItem('token_expiry');
    this.tokenExpiry = expiry ? parseInt(expiry) : null;
  }

  // Проверка истечения токена
  isTokenExpired() {
    if (!this.tokenExpiry) return true;
    return Date.now() > (this.tokenExpiry - 30000); // 30 секунд запас
  }

  // Обновление токена через Keycloak
  async refreshAccessToken() {
    if (!this.refreshToken) {
      throw new Error('No refresh token available');
    }

    const response = await fetch('http://localhost:8081/realms/kubeatlas/protocol/openid-connect/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded'
      },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        client_id: 'kubeatlas-backend',
        client_secret: 'backend-secret-key', // В продакшне используйте PKCE
        refresh_token: this.refreshToken
      })
    });

    if (!response.ok) {
      // Если refresh token тоже протух - логинимся заново
      this.clearTokens();
      window.location.href = '/login';
      throw new Error('Refresh token expired');
    }

    const data = await response.json();
    this.setTokens(data.access_token, data.refresh_token, data.expires_in);
    
    return data.access_token;
  }

  // Выполнение запроса с автоматическим обновлением токена
  async makeAuthenticatedRequest(url, options = {}) {
    // Проверяем нужно ли обновить токен
    if (this.isTokenExpired()) {
      console.log('Token expired, refreshing...');
      await this.refreshAccessToken();
    }

    // Выполняем запрос
    const response = await fetch(url, {
      ...options,
      headers: {
        'Authorization': `Bearer ${this.accessToken}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    // Если получили 401 - возможно токен протух между проверкой и запросом
    if (response.status === 401) {
      console.log('Got 401, attempting token refresh and retry...');
      await this.refreshAccessToken();
      
      // Повторяем запрос с новым токеном
      return fetch(url, {
        ...options,
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json',
          ...options.headers
        }
      });
    }

    return response;
  }

  // Очистка токенов (выход)
  clearTokens() {
    this.accessToken = null;
    this.refreshToken = null;
    this.tokenExpiry = null;
    localStorage.removeItem('access_token');
    localStorage.removeItem('refresh_token');
    localStorage.removeItem('token_expiry');
  }
}

export const authManager = new AuthManager();

// Инициализация при загрузке страницы
authManager.loadTokens();
```

### 🔧 **Интеграция с существующими функциями:**

Обновим наши предыдущие функции для использования правильного управления токенами:

```js
// Обновленная функция получения статистики
const getStatistics = async () => {
  try {
    // Используем tokenManager для автоматической обработки токена
    const response = await tokenManager.makeAuthenticatedRequest(
      'http://localhost:3001/api/v1/statistics'
    );
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const data = await response.json();
    
    if (data.success) {
      updateDashboardCards(data.data);
    } else {
      console.error('API error:', data.error);
    }
  } catch (error) {
    console.error('Failed to fetch statistics:', error);
    handleApiError(error);
  }
};

// Обновленная функция для работы с пользователями
const getAllUsers = async () => {
  try {
    const response = await tokenManager.makeAuthenticatedRequest(
      'http://localhost:3001/api/v1/admin/users'
    );
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const result = await response.json();
    return result.users;
  } catch (error) {
    console.error('Failed to fetch users:', error);
    throw error;
  }
};

// Универсальная функция для всех API вызовов
const apiCall = async (endpoint, options = {}) => {
  try {
    const response = await tokenManager.makeAuthenticatedRequest(
      `http://localhost:3001${endpoint}`,
      options
    );
    
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || `HTTP error! status: ${response.status}`);
    }
    
    return await response.json();
  } catch (error) {
    console.error(`API call failed for ${endpoint}:`, error);
    throw error;
  }
};

// Примеры использования универсальной функции
const getUserRoles = () => apiCall('/api/v1/user/roles');
const getStatisticsData = () => apiCall('/api/v1/statistics');
const createUser = (userData) => apiCall('/api/v1/admin/users', {
  method: 'POST',
  body: JSON.stringify(userData)
});
const deleteUser = (userId) => apiCall(`/api/v1/admin/users/${userId}`, {
  method: 'DELETE'
});
```

### ⏰ **Мониторинг токенов в реальном времени:**

```js
// Компонент для отображения статуса токена (для отладки)
const TokenStatus = () => {
  const [tokenInfo, setTokenInfo] = useState({
    timeLeft: 0,
    isExpired: false,
    lastRefresh: null
  });

  useEffect(() => {
    const updateTokenStatus = () => {
      setTokenInfo({
        timeLeft: tokenManager.getTokenTimeLeft(),
        isExpired: tokenManager.keycloak.isTokenExpired(),
        lastRefresh: new Date().toLocaleTimeString()
      });
    };

    // Обновляем каждую секунду
    const interval = setInterval(updateTokenStatus, 1000);
    updateTokenStatus(); // Инициальное обновление

    return () => clearInterval(interval);
  }, []);

  const formatTime = (seconds) => {
    const minutes = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${minutes}:${secs.toString().padStart(2, '0')}`;
  };

  return (
    <div className="token-status">
      <span className={tokenInfo.isExpired ? 'expired' : 'valid'}>
        Token: {tokenInfo.isExpired ? 'EXPIRED' : `${formatTime(tokenInfo.timeLeft)} left`}
      </span>
      {tokenInfo.lastRefresh && (
        <small>Last check: {tokenInfo.lastRefresh}</small>
      )}
    </div>
  );
};
```

## 5) Обработка ошибок и лучшие практики

### 🚨 **Критически важная обработка 401 ошибок:**

Важно правильно обрабатывать ошибку 401 (Unauthorized), которая может возникать когда токен протекает:

```js
// Универсальная функция обработки ошибок API
const handleApiError = async (error, response = null) => {
  console.error('API Error:', error);
  
  if (response) {
    switch (response.status) {
      case 401:
        console.warn('Token expired or invalid - attempting refresh');
        try {
          // Пытаемся обновить токен
          await tokenManager.refreshToken();
          // После успешного обновления можно показать уведомление
          showNotification('Session refreshed', 'success');
          return 'retry'; // Сигнал что нужно повторить запрос
        } catch (refreshError) {
          console.error('Token refresh failed:', refreshError);
          // Перенаправляем на логин
          keycloak.login();
          return 'login_required';
        }
        break;
        
      case 403:
        showError('Недостаточно прав для выполнения данного действия');
        break;
        
      case 500:
        showError('Ошибка сервера. Попробуйте позже.');
        break;
        
      case 429:
        showError('Слишком много запросов. Подождите немного.');
        break;
        
      default:
        showError(`Произошла ошибка (${response.status}). Попробуйте еще раз.`);
    }
  } else {
    // Сетевые ошибки
    if (error.name === 'NetworkError' || !navigator.onLine) {
      showError('Проблема с сетью. Проверьте подключение к интернету.');
    } else {
      showError('Произошла неожиданная ошибка.');
    }
  }
  
  return 'error';
};

// Продвинутая функция для API вызовов с retry логикой
const makeResilientApiCall = async (url, options = {}, maxRetries = 3) => {
  let retryCount = 0;
  
  while (retryCount < maxRetries) {
    try {
      const response = await tokenManager.makeAuthenticatedRequest(url, options);
      
      if (response.ok) {
        return await response.json();
      }
      
      // Если получили ошибку - обрабатываем
      const errorAction = await handleApiError(null, response);
      
      if (errorAction === 'retry' && retryCount < maxRetries - 1) {
        retryCount++;
        console.log(`Retrying request (${retryCount}/${maxRetries})...`);
        continue;
      }
      
      // Если это не retry ситуация - пробрасываем ошибку
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || `HTTP ${response.status}`);
      
    } catch (error) {
      retryCount++;
      
      if (retryCount >= maxRetries) {
        await handleApiError(error);
        throw error;
      }
      
      // Экспоненциальная задержка между попытками
      const delay = Math.min(1000 * Math.pow(2, retryCount - 1), 10000);
      console.log(`Request failed, retrying in ${delay}ms...`);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
};

// Пример использования устойчивых API вызовов
const getStatisticsResilient = async () => {
  try {
    showLoadingState();
    const data = await makeResilientApiCall('/api/v1/statistics');
    
    if (data.success) {
      updateDashboardCards(data.data);
    } else {
      throw new Error(data.error || 'API returned success: false');
    }
  } catch (error) {
    console.error('Failed to load statistics after all retries:', error);
    showError('Не удалось загрузить статистику. Попробуйте обновить страницу.');
  } finally {
    hideLoadingState();
  }
};
```

### Дополнительная обработка ошибок API:

```js
const handleApiError = (error, response) => {
  switch (response?.status) {
    case 401:
      // Неавторизован - перенаправляем на логин
      keycloak.login();
      break;
    case 403:
      // Недостаточно прав - показываем сообщение
      showError('Недостаточно прав для выполнения данного действия');
      break;
    case 500:
      // Ошибка сервера
      showError('Ошибка сервера. Попробуйте позже.');
      break;
    default:
      showError('Произошла ошибка. Попробуйте еще раз.');
  }
};
```

### Оптимизация производительности:

```js
// Кеширование статистики на 30 секунд
let statisticsCache = null;
let lastFetchTime = 0;
const CACHE_DURATION = 30000; // 30 секунд

const getCachedStatistics = async () => {
  const now = Date.now();
  
  if (statisticsCache && (now - lastFetchTime) < CACHE_DURATION) {
    return statisticsCache;
  }
  
  statisticsCache = await getStatistics();
  lastFetchTime = now;
  return statisticsCache;
};

// Прогрессивное отображение данных
const showLoadingState = () => {
  document.querySelectorAll('.stat-card').forEach(card => {
    card.classList.add('loading');
  });
};

const hideLoadingState = () => {
  document.querySelectorAll('.stat-card').forEach(card => {
    card.classList.remove('loading');
  });
};
```

### Пример CSS для карточек статистики:

```css
.stat-card {
  padding: 20px;
  border-radius: 8px;
  background: white;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  transition: all 0.3s ease;
}

.stat-card.loading {
  opacity: 0.6;
  pointer-events: none;
}

.stat-card.loading::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  width: 20px;
  height: 20px;
  border: 2px solid #f3f3f3;
  border-top: 2px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  transform: translate(-50%, -50%);
}

@keyframes spin {
  0% { transform: translate(-50%, -50%) rotate(0deg); }
  100% { transform: translate(-50%, -50%) rotate(360deg); }
}

.status-ok {
  color: #28a745;
  font-weight: 500;
}

.status-error {
  color: #dc3545;
  font-weight: 500;
}

.change-positive {
  color: #28a745;
}

.change-negative {
  color: #dc3545;
}
```

## 6) TypeScript типы для статистики

Для лучшей типизации в TypeScript проектах:

```typescript
// types/api.ts
export interface StatItem {
  value: number;
  change_percent: number;
  change_period: string;
}

export interface ServiceStatus {
  name: string;
  status: 'operational' | 'degraded' | 'outage';
  uptime_percentage: number;
}

export interface SystemStatus {
  percentage: number;
  status: string;
  details: ServiceStatus[];
}

export interface StatisticsResponse {
  total_users: StatItem;
  active_sessions: StatItem;
  system_status: SystemStatus;
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
}

// Типы для остальных API эндпоинтов
export interface UserProfile {
  id: string;
  username: string;
  email: string;
  first_name?: string;
  last_name?: string;
  roles: string[];
  is_admin: boolean;
  is_user: boolean;
  is_guest: boolean;
}

export interface UserRole {
  username: string;
  roles: string[];
  is_admin: boolean;
  is_user: boolean;
  is_guest: boolean;
}

export interface CreateUserRequest {
  username: string;
  email: string;
  first_name?: string;
  last_name?: string;
  password: string;
  roles: string[];
}

export interface UpdateUserRequest {
  email?: string;
  first_name?: string;
  last_name?: string;
  roles?: string[];
}

// Новые типы для управления пользователями и сессиями
export interface User {
  id: string;
  username: string;
  email: string;
  firstName?: string;
  lastName?: string;
  enabled: boolean;
  emailVerified: boolean;
  createdTimestamp: number;
  attributes?: Record<string, string[]>;
}

export interface UserSession {
  id: string;
  userId: string;
  username: string;
  ipAddress: string;
  start: number;
  lastAccess: number;
  clients: Record<string, string>;
}

export interface Role {
  id: string;
  name: string;
  description?: string;
  composite: boolean;
  clientRole: boolean;
  containerId: string;
}

export interface SessionRevocationResponse {
  success: boolean;
  message: string;
  sessionsRevoked?: number;
}

export interface UsersListResponse {
  users: User[];
  totalCount: number;
}

export interface UserDetailsResponse {
  user: User;
  roles: Role[];
  sessions: UserSession[];
}
```

Пример использования с TypeScript:

```typescript
// services/statistics.ts
import { ApiResponse, StatisticsResponse } from '../types/api';

export class StatisticsService {
  private readonly baseUrl = 'http://localhost:3001/api/v1';
  
  async getStatistics(): Promise<StatisticsResponse> {
    await keycloak.updateToken(30);
    
    const response = await fetch(`${this.baseUrl}/statistics`, {
      headers: {
        'Authorization': `Bearer ${keycloak.token}`,
        'Content-Type': 'application/json'
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }
    
    const data: ApiResponse<StatisticsResponse> = await response.json();
    
    if (!data.success || !data.data) {
      throw new Error(data.error || 'Ошибка получения статистики');
    }
    
    return data.data;
  }
}

// services/userManagement.ts
import { 
  User, 
  UserSession, 
  Role, 
  SessionRevocationResponse,
  CreateUserRequest,
  UpdateUserRequest,
  ApiResponse 
} from '../types/api';

export class UserManagementService {
  private readonly baseUrl = 'http://localhost:3001/api/v1';

  private async makeRequest<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    await keycloak.updateToken(30);
    
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${keycloak.token}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return response.json();
  }

  // Получение всех пользователей
  async getAllUsers(): Promise<User[]> {
    return this.makeRequest<User[]>('/admin/users');
  }

  // Получение пользователя по ID
  async getUserById(userId: string): Promise<User> {
    return this.makeRequest<User>(`/admin/users/${userId}`);
  }

  // Получение ролей пользователя
  async getUserRoles(userId: string): Promise<Role[]> {
    return this.makeRequest<Role[]>(`/admin/users/${userId}/roles`);
  }

  // Получение сессий пользователя
  async getUserSessions(userId: string): Promise<UserSession[]> {
    return this.makeRequest<UserSession[]>(`/admin/users/${userId}/sessions`);
  }

  // Создание пользователя
  async createUser(userData: CreateUserRequest): Promise<User> {
    return this.makeRequest<User>('/admin/users', {
      method: 'POST',
      body: JSON.stringify(userData)
    });
  }

  // Обновление пользователя
  async updateUser(userId: string, userData: UpdateUserRequest): Promise<User> {
    return this.makeRequest<User>(`/admin/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(userData)
    });
  }

  // Удаление пользователя
  async deleteUser(userId: string): Promise<void> {
    return this.makeRequest<void>(`/admin/users/${userId}`, {
      method: 'DELETE'
    });
  }

  // Закрытие всех сессий пользователя
  async revokeAllUserSessions(userId: string): Promise<SessionRevocationResponse> {
    return this.makeRequest<SessionRevocationResponse>(`/admin/users/${userId}/sessions/revoke`, {
      method: 'POST'
    });
  }

  // Закрытие конкретной сессии
  async revokeSpecificSession(userId: string, sessionId: string): Promise<SessionRevocationResponse> {
    return this.makeRequest<SessionRevocationResponse>(`/admin/users/${userId}/sessions/${sessionId}`, {
      method: 'DELETE'
    });
  }

  // Получение полной информации о пользователе (пользователь + роли + сессии)
  async getUserFullDetails(userId: string): Promise<{
    user: User;
    roles: Role[];
    sessions: UserSession[];
  }> {
    const [user, roles, sessions] = await Promise.all([
      this.getUserById(userId),
      this.getUserRoles(userId),
      this.getUserSessions(userId)
    ]);

    return { user, roles, sessions };
  }
}

// hooks/useStatistics.ts (для React)
import { useState, useEffect } from 'react';
import { StatisticsResponse } from '../types/api';
import { StatisticsService } from '../services/statistics';

const statisticsService = new StatisticsService();

export const useStatistics = (refreshInterval = 30000) => {
  const [statistics, setStatistics] = useState<StatisticsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  const fetchStatistics = async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await statisticsService.getStatistics();
      setStatistics(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Неизвестная ошибка');
    } finally {
      setLoading(false);
    }
  };
  
  useEffect(() => {
    fetchStatistics();
    
    const interval = setInterval(fetchStatistics, refreshInterval);
    return () => clearInterval(interval);
  }, [refreshInterval]);
  
  return { statistics, loading, error, refetch: fetchStatistics };
};

// hooks/useUserManagement.ts (для React)
import { useState, useEffect, useCallback } from 'react';
import { User, UserSession, Role } from '../types/api';
import { UserManagementService } from '../services/userManagement';

const userManagementService = new UserManagementService();

export const useUsers = () => {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchUsers = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await userManagementService.getAllUsers();
      setUsers(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Неизвестная ошибка');
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchUsers();
  }, [fetchUsers]);

  const deleteUser = async (userId: string) => {
    try {
      await userManagementService.deleteUser(userId);
      await fetchUsers(); // Обновляем список
    } catch (err) {
      throw err;
    }
  };

  return { users, loading, error, refetch: fetchUsers, deleteUser };
};

export const useUserDetails = (userId: string) => {
  const [user, setUser] = useState<User | null>(null);
  const [roles, setRoles] = useState<Role[]>([]);
  const [sessions, setSessions] = useState<UserSession[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchUserDetails = useCallback(async () => {
    if (!userId) return;
    
    try {
      setLoading(true);
      setError(null);
      const data = await userManagementService.getUserFullDetails(userId);
      setUser(data.user);
      setRoles(data.roles);
      setSessions(data.sessions);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Неизвестная ошибка');
    } finally {
      setLoading(false);
    }
  }, [userId]);

  useEffect(() => {
    fetchUserDetails();
  }, [fetchUserDetails]);

  const revokeSession = async (sessionId: string) => {
    try {
      await userManagementService.revokeSpecificSession(userId, sessionId);
      await fetchUserDetails(); // Обновляем данные
    } catch (err) {
      throw err;
    }
  };

  const revokeAllSessions = async () => {
    try {
      await userManagementService.revokeAllUserSessions(userId);
      await fetchUserDetails(); // Обновляем данные
    } catch (err) {
      throw err;
    }
  };

  return { 
    user, 
    roles, 
    sessions, 
    loading, 
    error, 
    refetch: fetchUserDetails,
    revokeSession,
    revokeAllSessions
  };
};
```

## 8) Обновленные рекомендации и лучшие практики

### Конфигурация токенов

Основная проблема решена - Keycloak теперь выдает токены с правильным issuer URL (`http://localhost:8081`). Это означает, что фронтенд может полноценно работать с бэкендом.

### Полный список доступных эндпоинтов:

**Основные эндпоинты:**
- `GET /health` - Проверка статуса сервиса
- `POST /auth/validate` - Валидация токена
- `GET /api/v1/statistics` - Статистика системы

**Пользовательские эндпоинты:**
- `GET /api/v1/user/profile` - Профиль пользователя
- `GET /api/v1/user/roles` - Роли пользователя

**Административные эндпоинты:**
- `GET /api/v1/admin/users` - Получение всех пользователей
- `GET /api/v1/admin/users/{id}` - Получение пользователя по ID
- `GET /api/v1/admin/users/{id}/roles` - Получение ролей пользователя
- `GET /api/v1/admin/users/{id}/sessions` - Получение сессий пользователя
- `POST /api/v1/admin/users` - Создание пользователя
- `PUT /api/v1/admin/users/{id}` - Обновление пользователя
- `DELETE /api/v1/admin/users/{id}` - Удаление пользователя
- `POST /api/v1/admin/users/{id}/sessions/revoke` - Закрытие всех сессий
- `DELETE /api/v1/admin/users/{id}/sessions/{session_id}` - Закрытие отдельной сессии (НОВОЕ!)

### Особенности работы с сессиями

1. **Индивидуальное закрытие сессий** - теперь возможно закрывать конкретные сессии по ID
2. **Отслеживание сессий** - можно получать подробную информацию о каждой сессии
3. **Массовое управление** - возможность закрыть все сессии пользователя одной командой

### Пример полного рабочего цикла

```js
// Полный пример работы с пользователями и сессиями
const userManagement = {
  // 1. Получаем всех пользователей
  async loadUsers() {
    const users = await userManagementService.getAllUsers();
    return users;
  },
  
  // 2. Получаем полную информацию о пользователе
  async getUserDetails(userId) {
    const details = await userManagementService.getUserFullDetails(userId);
    return details; // { user, roles, sessions }
  },
  
  // 3. Управляем сессиями
  async manageUserSessions(userId) {
    const sessions = await userManagementService.getUserSessions(userId);
    
    // Закрываем конкретную сессию
    if (sessions.length > 0) {
      await userManagementService.revokeSpecificSession(userId, sessions[0].id);
    }
    
    // Или закрываем все сессии
    await userManagementService.revokeAllUserSessions(userId);
  }
};
```

### Тестирование интеграции

Для проверки работоспособности вашего фронтенда, выполните следующие шаги:

1. **Проверка токенов:** Убедитесь, что issuer в токенах = `http://localhost:8081`
2. **Тест API:** Проверьте все ключевые эндпоинты
3. **Управление сессиями:** Проверьте закрытие сессий

Все новые функции реализованы и готовы к использованию!

---

## 9) 🚀 **Полные примеры интеграции с обработкой токенов**

### 🎡 **React Hook для полноценной работы с API:**

```jsx
// hooks/useKubeAtlasApi.js
import { useState, useCallback, useRef } from 'react';
import { tokenManager } from '../utils/tokenManager';

export const useKubeAtlasApi = () => {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);
  const abortControllerRef = useRef(null);

  // Основная функция для API вызовов
  const callApi = useCallback(async (endpoint, options = {}) => {
    // Отменяем предыдущие запросы
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    
    abortControllerRef.current = new AbortController();
    
    setLoading(true);
    setError(null);
    
    try {
      const response = await tokenManager.makeAuthenticatedRequest(
        `http://localhost:3001${endpoint}`,
        {
          ...options,
          signal: abortControllerRef.current.signal
        }
      );
      
      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || `HTTP ${response.status}`);
      }
      
      const data = await response.json();
      return data;
      
    } catch (err) {
      if (err.name === 'AbortError') {
        console.log('Request was aborted');
        return null;
      }
      
      setError(err.message);
      throw err;
      
    } finally {
      setLoading(false);
      abortControllerRef.current = null;
    }
  }, []);

  // Отмена текущих запросов
  const cancelRequest = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
  }, []);

  return {
    loading,
    error,
    callApi,
    cancelRequest,
    clearError: () => setError(null)
  };
};

// Специализированные hooks для конкретных операций
export const useUserManagement = () => {
  const { callApi, ...rest } = useKubeAtlasApi();

  const getAllUsers = useCallback(async () => {
    const result = await callApi('/api/v1/admin/users');
    return result?.users || [];
  }, [callApi]);

  const createUser = useCallback(async (userData) => {
    return callApi('/api/v1/admin/users', {
      method: 'POST',
      body: JSON.stringify(userData)
    });
  }, [callApi]);

  const deleteUser = useCallback(async (userId) => {
    return callApi(`/api/v1/admin/users/${userId}`, {
      method: 'DELETE'
    });
  }, [callApi]);

  const getUserSessions = useCallback(async (userId) => {
    const result = await callApi(`/api/v1/admin/users/${userId}/sessions`);
    return result?.sessions || [];
  }, [callApi]);

  const revokeUserSessions = useCallback(async (userId, sessionId = null) => {
    const endpoint = sessionId 
      ? `/api/v1/admin/users/${userId}/sessions/${sessionId}`
      : `/api/v1/admin/users/${userId}/sessions/revoke`;
    
    return callApi(endpoint, {
      method: sessionId ? 'DELETE' : 'POST'
    });
  }, [callApi]);

  return {
    ...rest,
    getAllUsers,
    createUser,
    deleteUser,
    getUserSessions,
    revokeUserSessions
  };
};

export const useStatistics = () => {
  const { callApi, ...rest } = useKubeAtlasApi();

  const getStatistics = useCallback(async () => {
    const result = await callApi('/api/v1/statistics');
    return result?.data || null;
  }, [callApi]);

  return {
    ...rest,
    getStatistics
  };
};

export const useUserProfile = () => {
  const { callApi, ...rest } = useKubeAtlasApi();

  const getProfile = useCallback(async () => {
    return callApi('/api/v1/user/profile');
  }, [callApi]);

  const getRoles = useCallback(async () => {
    return callApi('/api/v1/user/roles');
  }, [callApi]);

  return {
    ...rest,
    getProfile,
    getRoles
  };
};
```

### 📋 **Компонент Dashboard с авто-обновлением:**

```jsx
// components/Dashboard.jsx
import React, { useState, useEffect } from 'react';
import { useStatistics } from '../hooks/useKubeAtlasApi';

const Dashboard = () => {
  const [statistics, setStatistics] = useState(null);
  const [lastUpdate, setLastUpdate] = useState(null);
  const { getStatistics, loading, error } = useStatistics();

  // Автоматическое обновление каждые 30 секунд
  useEffect(() => {
    const loadStatistics = async () => {
      try {
        const data = await getStatistics();
        if (data) {
          setStatistics(data);
          setLastUpdate(new Date());
        }
      } catch (err) {
        console.error('Failed to load statistics:', err);
      }
    };

    // Начальная загрузка
    loadStatistics();

    // Периодическое обновление
    const interval = setInterval(loadStatistics, 30000);

    return () => clearInterval(interval);
  }, [getStatistics]);

  const formatNumber = (num) => {
    return new Intl.NumberFormat('ru-RU').format(num);
  };

  const formatLastUpdate = (date) => {
    return date ? date.toLocaleTimeString('ru-RU') : 'Никогда';
  };

  if (loading && !statistics) {
    return (
      <div className="dashboard-loading">
        <div className="spinner" />
        <p>Загрузка статистики...</p>
      </div>
    );
  }

  if (error && !statistics) {
    return (
      <div className="dashboard-error">
        <p>Ошибка загрузки: {error}</p>
        <button onClick={() => window.location.reload()}>
          Обновить страницу
        </button>
      </div>
    );
  }

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h1>Панель управления KubeAtlas</h1>
        <div className="last-update">
          Последнее обновление: {formatLastUpdate(lastUpdate)}
          {loading && <span className="updating"> (обновление...)</span>}
        </div>
      </div>

      {statistics && (
        <div className="stats-grid">
          {/* Карточка пользователей */}
          <div className="stat-card">
            <div className="stat-header">
              <h3>Общее количество пользователей</h3>
              <div className="stat-icon">👥</div>
            </div>
            <div className="stat-value">
              {formatNumber(statistics.total_users.value)}
            </div>
            <div className="stat-change positive">
              +{statistics.total_users.change_percent}% {statistics.total_users.change_period}
            </div>
          </div>

          {/* Карточка активных сессий */}
          <div className="stat-card">
            <div className="stat-header">
              <h3>Активные сессии</h3>
              <div className="stat-icon">🔗</div>
            </div>
            <div className="stat-value">
              {formatNumber(statistics.active_sessions.value)}
            </div>
            <div className="stat-change positive">
              +{statistics.active_sessions.change_percent}% {statistics.active_sessions.change_period}
            </div>
          </div>

          {/* Карточка статуса системы */}
          <div className="stat-card system-status">
            <div className="stat-header">
              <h3>Статус системы</h3>
              <div className="stat-icon">⚙️</div>
            </div>
            <div className="stat-value">
              {statistics.system_status.percentage.toFixed(1)}%
            </div>
            <div className="system-status-text">
              {statistics.system_status.status}
            </div>
            <div className="services-list">
              {statistics.system_status.details.map((service) => (
                <div 
                  key={service.name} 
                  className={`service-item ${service.status}`}
                >
                  <span className="service-name">{service.name}</span>
                  <span className="service-status">
                    {service.status} ({service.uptime_percentage}%)
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>
      )}

      {error && (
        <div className="error-banner">
          Ошибка при обновлении: {error}
        </div>
      )}
    </div>
  );
};

export default Dashboard;
```

### 🛠️ **Компонент управления пользователями:**

```jsx
// components/UserManagement.jsx
import React, { useState, useEffect } from 'react';
import { useUserManagement } from '../hooks/useKubeAtlasApi';

const UserManagement = () => {
  const [users, setUsers] = useState([]);
  const [selectedUser, setSelectedUser] = useState(null);
  const [userSessions, setUserSessions] = useState([]);
  const [showCreateForm, setShowCreateForm] = useState(false);
  
  const {
    getAllUsers,
    createUser,
    deleteUser,
    getUserSessions,
    revokeUserSessions,
    loading,
    error,
    clearError
  } = useUserManagement();

  // Загрузка списка пользователей
  useEffect(() => {
    loadUsers();
  }, []);

  const loadUsers = async () => {
    try {
      const usersList = await getAllUsers();
      setUsers(usersList);
    } catch (err) {
      console.error('Failed to load users:', err);
    }
  };

  const handleDeleteUser = async (userId) => {
    if (window.confirm('Вы уверены, что хотите удалить этого пользователя?')) {
      try {
        await deleteUser(userId);
        await loadUsers(); // Обновляем список
        alert('Пользователь успешно удален');
      } catch (err) {
        alert(`Ошибка удаления: ${err.message}`);
      }
    }
  };

  const handleViewSessions = async (user) => {
    try {
      setSelectedUser(user);
      const sessions = await getUserSessions(user.id);
      setUserSessions(sessions);
    } catch (err) {
      alert(`Ошибка загрузки сессий: ${err.message}`);
    }
  };

  const handleRevokeSession = async (sessionId) => {
    try {
      await revokeUserSessions(selectedUser.id, sessionId);
      // Обновляем список сессий
      const sessions = await getUserSessions(selectedUser.id);
      setUserSessions(sessions);
      alert('Сессия закрыта');
    } catch (err) {
      alert(`Ошибка закрытия сессии: ${err.message}`);
    }
  };

  const handleRevokeAllSessions = async () => {
    if (window.confirm(`Закрыть все сессии пользователя ${selectedUser.username}?`)) {
      try {
        await revokeUserSessions(selectedUser.id);
        // Обновляем список сессий
        const sessions = await getUserSessions(selectedUser.id);
        setUserSessions(sessions);
        alert('Все сессии закрыты');
      } catch (err) {
        alert(`Ошибка закрытия сессий: ${err.message}`);
      }
    }
  };

  return (
    <div className="user-management">
      <div className="user-management-header">
        <h1>Управление пользователями</h1>
        <button 
          className="btn-create"
          onClick={() => setShowCreateForm(true)}
        >
          Создать пользователя
        </button>
      </div>

      {error && (
        <div className="error-alert">
          {error}
          <button onClick={clearError}>×</button>
        </div>
      )}

      {loading && <div className="loading">Загрузка...</div>}

      <div className="users-table">
        <table>
          <thead>
            <tr>
              <th>Имя пользователя</th>
              <th>Email</th>
              <th>Имя</th>
              <th>Статус</th>
              <th>Действия</th>
            </tr>
          </thead>
          <tbody>
            {users.map((user) => (
              <tr key={user.id}>
                <td>{user.username}</td>
                <td>{user.email}</td>
                <td>{user.firstName} {user.lastName}</td>
                <td>
                  <span className={user.enabled ? 'enabled' : 'disabled'}>
                    {user.enabled ? 'Активный' : 'Отключен'}
                  </span>
                </td>
                <td>
                  <button 
                    className="btn-view-sessions"
                    onClick={() => handleViewSessions(user)}
                  >
                    Сессии
                  </button>
                  <button 
                    className="btn-delete"
                    onClick={() => handleDeleteUser(user.id)}
                  >
                    Удалить
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Модальное окно с сессиями пользователя */}
      {selectedUser && (
        <div className="modal-overlay" onClick={() => setSelectedUser(null)}>
          <div className="modal-content" onClick={e => e.stopPropagation()}>
            <div className="modal-header">
              <h2>Сессии пользователя {selectedUser.username}</h2>
              <button 
                className="btn-close"
                onClick={() => setSelectedUser(null)}
              >
                ×
              </button>
            </div>
            
            <div className="sessions-actions">
              <button 
                className="btn-revoke-all"
                onClick={handleRevokeAllSessions}
                disabled={userSessions.length === 0}
              >
                Закрыть все сессии ({userSessions.length})
              </button>
            </div>

            <div className="sessions-list">
              {userSessions.length === 0 ? (
                <p>Нет активных сессий</p>
              ) : (
                userSessions.map((session) => (
                  <div key={session.id} className="session-card">
                    <div className="session-info">
                      <p><strong>ID:</strong> {session.id}</p>
                      <p><strong>IP адрес:</strong> {session.ipAddress}</p>
                      <p><strong>Начало:</strong> {new Date(session.start).toLocaleString()}</p>
                      <p><strong>Последний доступ:</strong> {new Date(session.lastAccess).toLocaleString()}</p>
                    </div>
                    <button
                      className="btn-revoke"
                      onClick={() => handleRevokeSession(session.id)}
                    >
                      Закрыть
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default UserManagement;
```

### 📁 **Пример CSS стилей для компонентов:**

```css
/* styles/dashboard.css */
.dashboard {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
  padding-bottom: 15px;
  border-bottom: 2px solid #e5e5e5;
}

.dashboard-header h1 {
  color: #333;
  font-size: 2rem;
}

.last-update {
  color: #666;
  font-size: 0.9rem;
}

.updating {
  color: #007bff;
  font-weight: bold;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 20px;
  margin-bottom: 20px;
}

.stat-card {
  background: white;
  border-radius: 8px;
  padding: 20px;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  border: 1px solid #e5e5e5;
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.15);
}

.stat-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.stat-header h3 {
  margin: 0;
  color: #555;
  font-size: 1rem;
}

.stat-icon {
  font-size: 1.5rem;
}

.stat-value {
  font-size: 2.5rem;
  font-weight: bold;
  color: #333;
  margin-bottom: 10px;
}

.stat-change {
  font-size: 0.9rem;
  font-weight: 500;
}

.stat-change.positive {
  color: #28a745;
}

.stat-change.negative {
  color: #dc3545;
}

.system-status .services-list {
  margin-top: 15px;
}

.service-item {
  display: flex;
  justify-content: space-between;
  padding: 5px 0;
  border-bottom: 1px solid #f0f0f0;
}

.service-item:last-child {
  border-bottom: none;
}

.service-item.operational .service-status {
  color: #28a745;
}

.service-item.degraded .service-status {
  color: #ffc107;
}

.service-item.outage .service-status {
  color: #dc3545;
}

.error-banner {
  background: #f8d7da;
  color: #721c24;
  padding: 10px 15px;
  border-radius: 4px;
  margin-top: 20px;
  border: 1px solid #f5c6cb;
}

.dashboard-loading,
.dashboard-error {
  text-align: center;
  padding: 60px 20px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 20px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

/* styles/user-management.css */
.user-management {
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.user-management-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.btn-create,
.btn-view-sessions,
.btn-delete,
.btn-revoke,
.btn-revoke-all {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.9rem;
  transition: background-color 0.2s;
}

.btn-create {
  background: #28a745;
  color: white;
}

.btn-create:hover {
  background: #218838;
}

.btn-view-sessions {
  background: #007bff;
  color: white;
  margin-right: 10px;
}

.btn-view-sessions:hover {
  background: #0056b3;
}

.btn-delete {
  background: #dc3545;
  color: white;
}

.btn-delete:hover {
  background: #c82333;
}

.users-table {
  background: white;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0,0,0,0.1);
}

.users-table table {
  width: 100%;
  border-collapse: collapse;
}

.users-table th,
.users-table td {
  padding: 12px;
  text-align: left;
  border-bottom: 1px solid #dee2e6;
}

.users-table th {
  background: #f8f9fa;
  font-weight: 600;
  color: #495057;
}

.enabled {
  color: #28a745;
  font-weight: 500;
}

.disabled {
  color: #dc3545;
  font-weight: 500;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0,0,0,0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 8px;
  width: 90%;
  max-width: 600px;
  max-height: 80vh;
  overflow-y: auto;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 1px solid #e5e5e5;
}

.btn-close {
  background: none;
  border: none;
  font-size: 1.5rem;
  cursor: pointer;
  color: #666;
}

.sessions-actions {
  padding: 20px;
  border-bottom: 1px solid #e5e5e5;
}

.btn-revoke-all {
  background: #dc3545;
  color: white;
}

.btn-revoke-all:hover:not(:disabled) {
  background: #c82333;
}

.btn-revoke-all:disabled {
  background: #6c757d;
  cursor: not-allowed;
}

.sessions-list {
  padding: 20px;
}

.session-card {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 15px;
  border: 1px solid #e5e5e5;
  border-radius: 6px;
  margin-bottom: 10px;
}

.session-info p {
  margin: 5px 0;
  font-size: 0.9rem;
}

.btn-revoke {
  background: #ffc107;
  color: #212529;
}

.btn-revoke:hover {
  background: #e0a800;
}

.error-alert {
  background: #f8d7da;
  color: #721c24;
  padding: 10px 15px;
  border-radius: 4px;
  margin-bottom: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.error-alert button {
  background: none;
  border: none;
  font-size: 1.2rem;
  cursor: pointer;
  color: #721c24;
}

.loading {
  text-align: center;
  padding: 20px;
  color: #666;
}
```

### 📄 **Полный пример инициализации приложения:**

```js
// App.js - Основное приложение
import React, { useEffect, useState } from 'react';
import { keycloak, tokenManager } from './utils/tokenManager';
import Dashboard from './components/Dashboard';
import UserManagement from './components/UserManagement';
import './styles/dashboard.css';
import './styles/user-management.css';

const App = () => {
  const [authenticated, setAuthenticated] = useState(false);
  const [loading, setLoading] = useState(true);
  const [userRoles, setUserRoles] = useState([]);
  const [currentView, setCurrentView] = useState('dashboard');

  useEffect(() => {
    const initAuth = async () => {
      try {
        // Инициализация Keycloak
        const authenticated = await keycloak.init({
          onLoad: 'login-required',
          checkLoginIframe: false,
          pkceMethod: 'S256'
        });

        if (authenticated) {
          setAuthenticated(true);
          
          // Запускаем авто-обновление токенов
          tokenManager.startAutoRefresh();
          
          // Получаем роли пользователя
          const response = await tokenManager.makeAuthenticatedRequest('/api/v1/user/roles');
          if (response.ok) {
            const rolesData = await response.json();
            setUserRoles(rolesData.roles || []);
          }
        }
      } catch (error) {
        console.error('Authentication failed:', error);
      } finally {
        setLoading(false);
      }
    };

    initAuth();
  }, []);

  const isAdmin = userRoles.includes('admin');

  const handleLogout = () => {
    keycloak.logout();
  };

  if (loading) {
    return (
      <div className="app-loading">
        <div className="spinner" />
        <p>Инициализация...</p>
      </div>
    );
  }

  if (!authenticated) {
    return (
      <div className="app-error">
        <p>Ошибка аутентификации</p>
        <button onClick={() => keycloak.login()}>
          Войти снова
        </button>
      </div>
    );
  }

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-content">
          <h1>KubeAtlas</h1>
          <nav className="app-nav">
            <button 
              className={currentView === 'dashboard' ? 'nav-active' : ''}
              onClick={() => setCurrentView('dashboard')}
            >
              Панель управления
            </button>
            {isAdmin && (
              <button 
                className={currentView === 'users' ? 'nav-active' : ''}
                onClick={() => setCurrentView('users')}
              >
                Пользователи
              </button>
            )}
          </nav>
          <div className="user-menu">
            <span>Привет, {keycloak.tokenParsed?.preferred_username}!</span>
            <button onClick={handleLogout}>Выход</button>
          </div>
        </div>
      </header>

      <main className="app-main">
        {currentView === 'dashboard' && <Dashboard />}
        {currentView === 'users' && isAdmin && <UserManagement />}
      </main>
    </div>
  );
};

export default App;
```

### ⚙️ **Настройки для продакшна:**

1. **Увеличьте время жизни токенов** в Keycloak:
   - Access Token Lifespan: 15-30 минут (вместо 1 часа)
   - SSO Session Idle: 30 минут
   - SSO Session Max: 10 часов

2. **Настройте HTTPS** для продакшна

3. **Используйте переменные окружения** для URLи:

```js
// config/env.js
export const API_CONFIG = {
  BASE_URL: process.env.REACT_APP_API_URL || 'http://localhost:3001',
  KEYCLOAK_URL: process.env.REACT_APP_KEYCLOAK_URL || 'http://localhost:8081',
  KEYCLOAK_REALM: process.env.REACT_APP_KEYCLOAK_REALM || 'kubeatlas',
  KEYCLOAK_CLIENT_ID: process.env.REACT_APP_KEYCLOAK_CLIENT_ID || 'kubeatlas-backend'
};
```

### 📈 **Мониторинг и отладка:**

Для отладки проблем с токенами добавьте в консоль браузера:

```js
// Отладочная информация о токене
console.log('Token Info:', {
  token: keycloak.token?.substring(0, 50) + '...',
  isExpired: keycloak.isTokenExpired(),
  timeLeft: tokenManager.getTokenTimeLeft(),
  username: keycloak.tokenParsed?.preferred_username,
  roles: keycloak.tokenParsed?.realm_access?.roles
});
```

Теперь у вас есть полноценная система работы с KubeAtlas API, которая:

✅ **Автоматически обновляет токены**  
✅ **Повторяет запросы при 401 ошибке**  
✅ **Обрабатывает все виды ошибок**  
✅ **Предоставляет удобные React hooks**  
✅ **Поддерживает TypeScript**
