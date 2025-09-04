#!/bin/bash

echo "Тестирование функциональности удаления пользователей"
echo "=================================================="

# Проверяем здоровье сервиса
echo "1. Проверка здоровья backend сервиса..."
HEALTH=$(curl -s http://localhost:3001/health | jq -r '.status')
echo "Статус: $HEALTH"

if [ "$HEALTH" != "healthy" ]; then
    echo "❌ Backend сервис недоступен"
    exit 1
fi

# Получаем токен администратора
echo -e "\n2. Получение токена администратора..."
TOKEN_RESPONSE=$(curl -s -X POST http://localhost:8081/realms/kubeatlas/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=kubeatlas-backend" \
  -d "client_secret=backend-secret-key" \
  -d "username=admin-service" \
  -d "password=AdminPassw0rd!")

ADMIN_TOKEN=$(echo $TOKEN_RESPONSE | jq -r '.access_token')

if [ "$ADMIN_TOKEN" = "null" ] || [ -z "$ADMIN_TOKEN" ]; then
    echo "❌ Не удалось получить токен администратора"
    echo "Ответ: $TOKEN_RESPONSE"
    exit 1
fi

echo "✅ Токен получен: ${ADMIN_TOKEN:0:30}..."

# Создаем тестового пользователя
echo -e "\n3. Создание тестового пользователя..."
CREATE_RESPONSE=$(curl -s -X POST http://localhost:3001/api/v1/admin/users \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_user_for_deletion",
    "email": "test_user_for_deletion@example.com", 
    "first_name": "Test",
    "last_name": "User",
    "password": "TestPassword123!",
    "roles": ["user"]
  }')

echo "Результат создания: $CREATE_RESPONSE"

USER_ID=$(echo $CREATE_RESPONSE | jq -r '.id')

if [ "$USER_ID" = "null" ] || [ -z "$USER_ID" ]; then
    echo "❌ Не удалось создать тестового пользователя"
    exit 1
fi

echo "✅ Пользователь создан с ID: $USER_ID"

# Удаляем пользователя
echo -e "\n4. Удаление пользователя..."
DELETE_RESPONSE=$(curl -s -X DELETE "http://localhost:3001/api/v1/admin/users/$USER_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

echo "Результат удаления: $DELETE_RESPONSE"

# Проверяем, что пользователь действительно удален
echo -e "\n5. Проверка удаления (повторная попытка удаления должна вернуть ошибку)..."
VERIFY_RESPONSE=$(curl -s -X DELETE "http://localhost:3001/api/v1/admin/users/$USER_ID" \
  -H "Authorization: Bearer $ADMIN_TOKEN")

echo "Результат проверки: $VERIFY_RESPONSE"

echo -e "\n✅ Тестирование завершено!"
echo "=============================="