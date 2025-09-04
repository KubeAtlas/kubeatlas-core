# Интеграция управления пользователями с фронтендом

## 📋 Обзор

Этот документ описывает, как интегрировать функциональность управления пользователями KubeAtlas Backend с вашим фронтенд-приложением. Включает создание, обновление, удаление пользователей и управление сессиями.

## 🔒 Требования безопасности

### Проверка ролей на фронтенде

Перед отображением административных интерфейсов проверьте роли пользователя:

```javascript
// Проверка наличия админских прав
async function checkAdminRights() {
  try {
    const response = await fetch('http://localhost:3001/api/v1/user/roles', {
      headers: {
        'Authorization': `Bearer ${keycloak.token}`
      }
    });
    
    const data = await response.json();
    return data.roles.includes('admin');
  } catch (error) {
    console.error('Ошибка проверки ролей:', error);
    return false;
  }
}
```

## 🛠️ JavaScript/TypeScript примеры

### 1. Служба управления пользователями

```typescript
interface User {
  id?: string;
  username: string;
  email: string;
  first_name: string;
  last_name: string;
  password?: string;
  roles: string[];
}

interface ApiResponse<T> {
  success?: boolean;
  data?: T;
  message?: string;
  error?: string;
}

class UserManagementService {
  private baseUrl = 'http://localhost:3001/api/v1/admin';
  
  private async makeRequest<T>(
    endpoint: string, 
    options: RequestInit = {}
  ): Promise<T> {
    // Обновляем токен перед каждым запросом
    await keycloak.updateToken(30).catch(() => keycloak.login());
    
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        'Authorization': `Bearer ${keycloak.token}`,
        'Content-Type': 'application/json',
        ...options.headers
      }
    });
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    return response.json();
  }
  
  // Создание пользователя
  async createUser(user: User): Promise<{id: string}> {
    return this.makeRequest('/users', {
      method: 'POST',
      body: JSON.stringify(user)
    });
  }
  
  // Обновление пользователя
  async updateUser(userId: string, updates: Partial<User>): Promise<{id: string}> {
    return this.makeRequest(`/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(updates)
    });
  }
  
  // Удаление пользователя
  async deleteUser(userId: string): Promise<{message: string, id: string}> {
    return this.makeRequest(`/users/${userId}`, {
      method: 'DELETE'
    });
  }
  
  // Получение активных сессий пользователя
  async getUserSessions(userId: string): Promise<{sessions: any[]}> {
    return this.makeRequest(`/users/${userId}/sessions`);
  }
  
  // Отзыв всех сессий пользователя
  async revokeUserSessions(userId: string): Promise<{message: string}> {
    return this.makeRequest(`/users/${userId}/sessions/revoke`, {
      method: 'POST'
    });
  }
}

// Инициализация сервиса
const userService = new UserManagementService();
```

### 2. React компоненты

#### Компонент создания пользователя

```tsx
import React, { useState } from 'react';

interface CreateUserFormProps {
  onUserCreated: (userId: string) => void;
  onError: (error: string) => void;
}

export const CreateUserForm: React.FC<CreateUserFormProps> = ({ 
  onUserCreated, 
  onError 
}) => {
  const [formData, setFormData] = useState({
    username: '',
    email: '',
    first_name: '',
    last_name: '',
    password: '',
    roles: ['user']
  });
  
  const [loading, setLoading] = useState(false);
  
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    
    try {
      const result = await userService.createUser(formData);
      onUserCreated(result.id);
      setFormData({
        username: '',
        email: '',
        first_name: '',
        last_name: '',
        password: '',
        roles: ['user']
      });
    } catch (error) {
      onError(`Ошибка создания пользователя: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };
  
  return (
    <form onSubmit={handleSubmit} className="user-form">
      <div className="form-group">
        <label>Имя пользователя:</label>
        <input
          type="text"
          value={formData.username}
          onChange={(e) => setFormData({...formData, username: e.target.value})}
          required
        />
      </div>
      
      <div className="form-group">
        <label>Email:</label>
        <input
          type="email"
          value={formData.email}
          onChange={(e) => setFormData({...formData, email: e.target.value})}
          required
        />
      </div>
      
      <div className="form-group">
        <label>Имя:</label>
        <input
          type="text"
          value={formData.first_name}
          onChange={(e) => setFormData({...formData, first_name: e.target.value})}
          required
        />
      </div>
      
      <div className="form-group">
        <label>Фамилия:</label>
        <input
          type="text"
          value={formData.last_name}
          onChange={(e) => setFormData({...formData, last_name: e.target.value})}
          required
        />
      </div>
      
      <div className="form-group">
        <label>Пароль:</label>
        <input
          type="password"
          value={formData.password}
          onChange={(e) => setFormData({...formData, password: e.target.value})}
          required
          minLength={8}
        />
      </div>
      
      <div className="form-group">
        <label>Роли:</label>
        <select 
          multiple 
          value={formData.roles}
          onChange={(e) => setFormData({
            ...formData, 
            roles: Array.from(e.target.selectedOptions, option => option.value)
          })}
        >
          <option value="user">User</option>
          <option value="admin">Admin</option>
        </select>
      </div>
      
      <button type="submit" disabled={loading}>
        {loading ? 'Создание...' : 'Создать пользователя'}
      </button>
    </form>
  );
};
```

#### Компонент для удаления пользователя

```tsx
import React, { useState } from 'react';

interface DeleteUserButtonProps {
  userId: string;
  username: string;
  onUserDeleted: (userId: string) => void;
  onError: (error: string) => void;
}

export const DeleteUserButton: React.FC<DeleteUserButtonProps> = ({
  userId,
  username,
  onUserDeleted,
  onError
}) => {
  const [showConfirm, setShowConfirm] = useState(false);
  const [loading, setLoading] = useState(false);
  
  const handleDelete = async () => {
    setLoading(true);
    
    try {
      await userService.deleteUser(userId);
      onUserDeleted(userId);
      setShowConfirm(false);
    } catch (error) {
      onError(`Ошибка удаления пользователя: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };
  
  if (showConfirm) {
    return (
      <div className="delete-confirmation">
        <p>
          Вы уверены, что хотите удалить пользователя <strong>{username}</strong>?
          <br />
          <em>Это действие необратимо!</em>
        </p>
        <button 
          onClick={handleDelete} 
          disabled={loading}
          className="btn-danger"
        >
          {loading ? 'Удаление...' : 'Да, удалить'}
        </button>
        <button 
          onClick={() => setShowConfirm(false)}
          className="btn-cancel"
        >
          Отмена
        </button>
      </div>
    );
  }
  
  return (
    <button 
      onClick={() => setShowConfirm(true)}
      className="btn-delete"
    >
      Удалить
    </button>
  );
};
```

### 3. Управление сессиями

```tsx
import React, { useState, useEffect } from 'react';

interface UserSessionsProps {
  userId: string;
}

export const UserSessions: React.FC<UserSessionsProps> = ({ userId }) => {
  const [sessions, setSessions] = useState([]);
  const [loading, setLoading] = useState(false);
  
  const loadSessions = async () => {
    setLoading(true);
    try {
      const result = await userService.getUserSessions(userId);
      setSessions(result.sessions);
    } catch (error) {
      console.error('Ошибка загрузки сессий:', error);
    } finally {
      setLoading(false);
    }
  };
  
  const revokeSessions = async () => {
    try {
      await userService.revokeUserSessions(userId);
      await loadSessions(); // Перезагружаем список
    } catch (error) {
      console.error('Ошибка отзыва сессий:', error);
    }
  };
  
  useEffect(() => {
    loadSessions();
  }, [userId]);
  
  return (
    <div className="user-sessions">
      <h3>Активные сессии</h3>
      
      {loading && <p>Загрузка...</p>}
      
      {sessions.length > 0 && (
        <div>
          <p>Количество активных сессий: {sessions.length}</p>
          <button onClick={revokeSessions} className="btn-warning">
            Отозвать все сессии
          </button>
        </div>
      )}
      
      {sessions.length === 0 && !loading && (
        <p>Нет активных сессий</p>
      )}
    </div>
  );
};
```

## ⚠️ Важные замечания

### Обработка ошибок

```javascript
// Пример централизованной обработки ошибок
async function handleApiCall(apiFunction) {
  try {
    return await apiFunction();
  } catch (error) {
    if (error.message.includes('401')) {
      // Токен истек, перенаправляем на логин
      keycloak.login();
    } else if (error.message.includes('403')) {
      // Недостаточно прав
      alert('У вас недостаточно прав для выполнения этого действия');
    } else {
      // Другие ошибки
      console.error('API Error:', error);
      alert(`Ошибка: ${error.message}`);
    }
    throw error;
  }
}
```

### Валидация данных

```javascript
// Валидация данных пользователя
function validateUserData(user) {
  const errors = [];
  
  if (!user.username || user.username.length < 3) {
    errors.push('Имя пользователя должно содержать минимум 3 символа');
  }
  
  if (!user.email || !/\S+@\S+\.\S+/.test(user.email)) {
    errors.push('Введите корректный email');
  }
  
  if (!user.password || user.password.length < 8) {
    errors.push('Пароль должен содержать минимум 8 символов');
  }
  
  if (!user.first_name || !user.last_name) {
    errors.push('Имя и фамилия обязательны');
  }
  
  return errors;
}
```

## 🎨 CSS стили (пример)

```css
.user-form {
  max-width: 500px;
  margin: 0 auto;
  padding: 20px;
}

.form-group {
  margin-bottom: 15px;
}

.form-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.form-group input, 
.form-group select {
  width: 100%;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.btn-danger {
  background-color: #dc3545;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
}

.btn-danger:hover {
  background-color: #c82333;
}

.btn-cancel {
  background-color: #6c757d;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  cursor: pointer;
  margin-left: 10px;
}

.delete-confirmation {
  padding: 15px;
  border: 1px solid #dc3545;
  border-radius: 4px;
  background-color: #f8d7da;
}
```

## 📱 Мобильная адаптация

При разработке учитывайте мобильные устройства:

- Используйте подтверждающие диалоги для критических действий (удаление)
- Добавляйте индикаторы загрузки для длительных операций
- Обеспечьте доступность через aria-атрибуты
- Тестируйте на различных размерах экранов

## 🔄 Обновление токенов

```javascript
// Автоматическое обновление токена
setInterval(async () => {
  try {
    await keycloak.updateToken(300); // обновляем за 5 минут до истечения
  } catch (error) {
    console.log('Токен истек, требуется повторный вход');
    keycloak.login();
  }
}, 60000); // проверяем каждую минуту
```

Этот документ предоставляет полный набор инструментов для интеграции функциональности управления пользователями с любым современным фронтенд-фреймворком.
