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
