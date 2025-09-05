# Интеграция фронтенда с KubeAtlas Backend + Keycloak

## Содержание

1. [Конфигурация Keycloak на фронтенде](#1-конфигурация-keycloak-на-фронтенде)
2. [Передача токена в бэкенд](#2-передача-токена-в-бэкенд)
3. [Примеры вызовов API](#3-пример-вызовов)
   - [Пользовательские эндпоинты](#пользовательские-эндпоинты-требуют-аутентификации)
   - [Эндпоинт статистики (новый)](#статистика-системы-новый-эндпоинт)
   - [Административные эндпоинты](#административные-эндпоинты-требуют-роль-admin)
4. [Обновление токена](#4-обновление-токена)
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

## 5) Обработка ошибок и лучшие практики

### Обработка ошибок API:

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
```

## 7) CORS

Бэкенд уже включает CORS (allow origin: Any). Для продакшена задайте разрешённые домены.
