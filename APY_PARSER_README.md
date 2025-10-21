# APY Parser Module

## 📋 Описание

APY Parser - это модуль для автоматического сбора, хранения и предоставления исторических данных по APY (Annual Percentage Yield) различных DeFi протоколов, а также управления позициями пользователей для автоматического ребалансирования.

## 🏗️ Архитектура

### Основные компоненты

1. **APY Collection Engine** - периодический сбор APY из протоколов
2. **Position Manager** - управление позициями пользователей
3. **Persistent Storage** - StableBTreeMap для хранения данных
4. **Scheduler Integration** - интеграция с модулем автоматического ребалансирования

### Структуры данных

#### UserPosition
```rust
pub struct UserPosition {
    pub position_id: String,           // Уникальный ID позиции
    pub user_principal: Principal,     // Principal пользователя
    pub user_evm_address: String,      // EVM адрес пользователя
    pub permissions_id: String,        // ID разрешений
    pub protocol: String,              // "AAVE" | "COMPOUND"
    pub asset: String,                 // "USDC", "LINK", etc.
    pub token_address: String,         // EVM адрес токена
    pub chain_id: u64,                 // ID сети (42161 = Arbitrum)
    pub position_size: String,         // Размер позиции (human-readable)
    pub tracked: bool,                 // Отслеживать для ребалансировки?
    pub added_at: u64,                 // Timestamp создания
    pub updated_at: u64,               // Timestamp обновления
}
```

#### ApyHistoryRecord
```rust
pub struct ApyHistoryRecord {
    pub record_id: String,             // Уникальный ID записи
    pub protocol: String,              // "AAVE" | "COMPOUND"
    pub asset: String,                 // "USDC", "LINK", etc.
    pub token_address: String,         // EVM адрес токена
    pub chain_id: u64,                 // ID сети
    pub apy: f64,                      // APY в процентах
    pub timestamp: u64,                // Timestamp записи
}
```

#### ApyParserConfig
```rust
pub struct ApyParserConfig {
    pub enabled: bool,                 // Включен ли сборщик
    pub interval_seconds: u64,         // Интервал сбора (по умолчанию 900 = 15 минут)
    pub last_execution: Option<u64>,  // Время последнего сбора
    pub monitored_protocols: Vec<String>, // ["AAVE", "COMPOUND"]
    pub monitored_chains: Vec<u64>,    // [42161, 11155111]
}
```

## 🗄️ Хранилища (StableBTreeMap)

### APY_HISTORY_MAP
- **Ключ:** `record_id` (String)
- **Значение:** `ApyHistoryRecord`
- **Формат ключа:** `{protocol}:{chain_id}:{token_address}:{timestamp}`
- **Пример:** `AAVE:42161:0xaf88d065e77c8cC2239327C5EDb3A432268e5831:1704067200000`

### USER_POSITIONS_MAP
- **Ключ:** `position_id` (String)
- **Значение:** `UserPosition`
- **Формат ключа:** `pos_{timestamp_hex}{random_hex}`
- **Пример:** `pos_0000018d1234abcd5678ef90`

### REBALANCE_HISTORY_MAP
- **Ключ:** `execution_id` (String)
- **Значение:** `RebalanceExecution`
- **Назначение:** История всех выполненных ребалансировок

## 🔧 API Эндпоинты

### Пользовательские API

#### `create_position`
Создание новой позиции для отслеживания.

```bash
dfx canister call yieldex-ic-wallet-manager-backend create_position '(
  "permissions_id",      # ID разрешений пользователя
  "AAVE",               # Протокол
  "USDC",               # Символ токена
  "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", # Адрес токена
  42161,                # Chain ID (Arbitrum)
  "1000",               # Размер позиции в USDC
  true                  # Отслеживать для автоматического ребалансирования
)'
```

**Возвращает:** `UserPosition`

#### `get_my_positions`
Получение всех позиций текущего пользователя.

```bash
dfx canister call yieldex-ic-wallet-manager-backend get_my_positions
```

**Возвращает:** `Vec<UserPosition>`

#### `update_position`
Обновление параметров позиции.

```bash
dfx canister call yieldex-ic-wallet-manager-backend update_position '(
  "pos_0000018d1234abcd", # Position ID
  opt "2000",             # Новый размер позиции (optional)
  opt false               # Отключить отслеживание (optional)
)'
```

**Возвращает:** `Result<UserPosition, String>`

#### `delete_position`
Удаление позиции.

```bash
dfx canister call yieldex-ic-wallet-manager-backend delete_position '("pos_0000018d1234abcd")'
```

**Возвращает:** `Result<bool, String>`

#### `get_position`
Получение конкретной позиции по ID (только если принадлежит вызывающему пользователю).

```bash
dfx canister call yieldex-ic-wallet-manager-backend get_position '("pos_0000018d1234abcd")'
```

**Возвращает:** `Result<UserPosition, String>`

---

### Админские API

#### `admin_init_apy_parser`
Инициализация APY parser (для существующих канистеров, развернутых до добавления модуля).

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_init_apy_parser
```

**Возвращает:** `Result<String, String>`

#### `admin_start_apy_parser`
Запуск периодического сбора APY.

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_start_apy_parser
```

**Возвращает:** `Result<String, String>`

#### `admin_stop_apy_parser`
Остановка сбора APY.

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_stop_apy_parser
```

**Возвращает:** `Result<String, String>`

#### `admin_set_apy_parser_interval`
Настройка интервала сбора APY (в секундах).

```bash
# Установить интервал 15 минут (900 секунд)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_parser_interval '(900)'

# Установить интервал 1 час (3600 секунд)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_parser_interval '(3600)'
```

**Возвращает:** `Result<String, String>`

**Ограничения:** Минимум 60 секунд

#### `admin_trigger_apy_collection`
Ручной запуск сбора APY (не дожидаясь таймера).

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_trigger_apy_collection
```

**Возвращает:** `Result<String, String>`

#### `admin_get_apy_history`
Получение истории APY для конкретного протокола/токена/сети.

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_get_apy_history '(
  "AAVE",    # Протокол
  "USDC",    # Токен
  42161,     # Chain ID
  opt 10     # Лимит записей (optional, по умолчанию 100)
)'
```

**Возвращает:** `Vec<ApyHistoryRecord>` (отсортировано по убыванию timestamp)

#### `admin_get_all_positions`
Получение всех позиций в системе (всех пользователей).

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_get_all_positions
```

**Возвращает:** `Vec<UserPosition>`

#### `admin_get_tracked_positions`
Получение только отслеживаемых позиций (tracked = true).

```bash
dfx canister call yieldex-ic-wallet-manager-backend admin_get_tracked_positions
```

**Возвращает:** `Vec<UserPosition>`

## 🔄 Интеграция со Scheduler

APY Parser интегрирован с модулем автоматического ребалансирования:

1. **Scheduler использует APY Parser для получения позиций:**
   ```rust
   let tracked_positions = apy_parser::get_tracked_positions();
   ```

2. **Scheduler использует кэшированные APY данные:**
   ```rust
   let apy = apy_parser::get_latest_apy(protocol, asset, chain_id).await?;
   ```
   - Сначала проверяет кэш в `APY_HISTORY_MAP`
   - Если данных нет или они устарели, делает live запрос к протоколу

3. **История ребалансировок сохраняется в `REBALANCE_HISTORY_MAP`**

## 🚀 Быстрый старт

### 1. Развертывание и инициализация

```bash
# Развернуть канистер
dfx deploy yieldex-ic-wallet-manager-backend

# APY Parser автоматически инициализируется в init()
# Но нужно запустить его вручную
dfx canister call yieldex-ic-wallet-manager-backend admin_start_apy_parser
```

### 2. Настройка (опционально)

```bash
# Установить интервал сбора APY (например, 30 минут)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_parser_interval '(1800)'
```

### 3. Создание позиции пользователем

```bash
# 1. Генерация EVM адреса (если еще не создан)
dfx canister call yieldex-ic-wallet-manager-backend generate_evm_address

# 2. Создание разрешений (permissions)
dfx canister call yieldex-ic-wallet-manager-backend create_permissions '(record {
  chain_id = 42161;
  whitelisted_protocols = vec { record { name = "AAVE"; address = "0x794a61358D6845594F94dc1DB02A252b5b4814aD" } };
  whitelisted_tokens = vec { record { name = "USDC"; address = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831" } };
  transfer_limits = vec {};
  protocol_permissions = null;
})'

# 3. Создание позиции
dfx canister call yieldex-ic-wallet-manager-backend create_position '(
  "<permissions_id>",
  "AAVE",
  "USDC",
  "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
  42161,
  "1000",
  true
)'
```

### 4. Запуск scheduler для автоматического ребалансирования

```bash
# Запустить scheduler
dfx canister call yieldex-ic-wallet-manager-backend admin_start_scheduler

# Настроить порог APY для ребалансировки (например, 0.5%)
dfx canister call yieldex-ic-wallet-manager-backend admin_set_apy_threshold '(0.5)'
```

## 📊 Мониторинг

### Проверка статуса APY Parser

```bash
# Посмотреть последние собранные данные
dfx canister call yieldex-ic-wallet-manager-backend admin_get_apy_history '("AAVE", "USDC", 42161, opt 5)'

# Посмотреть все отслеживаемые позиции
dfx canister call yieldex-ic-wallet-manager-backend admin_get_tracked_positions

# Проверить статус scheduler (включает информацию о позициях)
dfx canister call yieldex-ic-wallet-manager-backend admin_get_scheduler_status
```

### Ручная проверка APY

```bash
# Получить текущий APY напрямую из протокола (admin only)
dfx canister call yieldex-ic-wallet-manager-backend get_current_apy '("USDC", 42161)'
```

## 🔍 Поддерживаемые протоколы и токены

### AAVE V3

| Сеть | Chain ID | Токены | Pool Address |
|------|----------|---------|--------------|
| Arbitrum | 42161 | USDC | 0x794a61358D6845594F94dc1DB02A252b5b4814aD |
| Sepolia | 11155111 | USDC | 0x6Ae43d3271ff6888e7Fc43Fd7321a503ff738951 |
| Base | 8453 | USDC | 0x794a61358D6845594F94dc1DB02A252b5b4814aD |
| Optimism | 10 | USDC | 0x794a61358D6845594F94dc1DB02A252b5b4814aD |

### Compound III

| Сеть | Chain ID | Токены | Comet Address |
|------|----------|---------|---------------|
| Arbitrum | 42161 | USDC | 0x9c4ec768c28520b50860ea7a15bd7213a9ff58bf |

## 🔐 Безопасность

### Контроль доступа

- **Пользовательские эндпоинты:** Доступны только владельцу позиций
- **Админские эндпоинты:** Проверка `is_admin()` через список `ADMIN_PRINCIPALS`

### Валидация данных

- Проверка ownership при обновлении/удалении позиций
- Проверка permissions перед созданием позиции
- Минимальный интервал сбора APY: 60 секунд

## 🐛 Отладка

### Логирование

APY Parser выводит подробные логи через `ic_cdk::println!`:

```
📊 APY collection started at 1704067200000
📊 Collecting APY for AAVE on chain 42161
  Fetching APY for USDC (0xaf88...) on AAVE
  ✅ Stored APY: 5.23% for USDC on AAVE
✅ Collected 1 APY records for AAVE on chain 42161
📋 APY Collection Summary:
  - Total records collected: 1
  - Errors: 0
✅ APY collection completed
```

### Распространенные ошибки

**"Scheduler not initialized"**
- Решение: Вызвать `admin_init_scheduler()` или `admin_init_apy_parser()`

**"Position ID cannot be empty"**
- Решение: Убедиться что все обязательные поля заполнены при создании позиции

**"Interval must be at least 60 seconds"**
- Решение: Установить интервал >= 60 секунд

**"Token X not found for protocol Y"**
- Решение: Убедиться что токен поддерживается протоколом на данной сети

## 📈 Производительность

### Рекомендуемые настройки

- **Интервал APY сбора:** 15-30 минут (900-1800 секунд)
- **Интервал scheduler:** 1-2 часа (3600-7200 секунд)
- **APY threshold для ребалансировки:** 0.5-1.0%

### Ограничения

- StableBTreeMap - unbounded для `UserPosition` и `ApyHistoryRecord`
- Рекомендуется периодическая очистка старых APY записей (можно добавить админский endpoint)

## 🔄 Обновление канистера

APY Parser корректно обрабатывает обновления канистера:

```rust
#[post_upgrade]
fn post_upgrade() {
    // Stable memory автоматически сохраняется

    // Восстанавливаются таймеры если были включены
    if apy_parser::is_apy_parser_enabled() {
        apy_parser::start_apy_parser_timer();
    }
}
```

## 📝 TODO / Будущие улучшения

- [ ] Добавить endpoint для очистки старых APY записей
- [ ] Поддержка большего количества токенов (ETH, DAI, USDT)
- [ ] Webhook уведомления при значительном изменении APY
- [ ] Графический дашборд для мониторинга APY
- [ ] Экспорт истории APY в CSV
- [ ] Автоматическая детекция позиций из on-chain данных

## 🤝 Интеграция с другими модулями

### Scheduler Module
- Использует `get_tracked_positions()` для получения позиций
- Использует `get_latest_apy()` для принятия решений о ребалансировке

### Rebalance Module
- История ребалансировок сохраняется в `REBALANCE_HISTORY_MAP`
- Доступна через `admin_get_rebalance_history()`

### Permissions Module
- Проверка permissions при создании позиций
- Валидация ownership

## 📚 Примеры использования

### Сценарий 1: Создание и отслеживание позиции

```bash
# 1. Создать позицию
POSITION=$(dfx canister call yieldex-ic-wallet-manager-backend create_position '(
  "perm_123",
  "AAVE",
  "USDC",
  "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
  42161,
  "5000",
  true
)' | grep position_id | awk '{print $3}')

# 2. Проверить позицию
dfx canister call yieldex-ic-wallet-manager-backend get_position "(\"$POSITION\")"

# 3. Обновить размер позиции
dfx canister call yieldex-ic-wallet-manager-backend update_position "(
  \"$POSITION\",
  opt \"7500\",
  null
)"
```

### Сценарий 2: Мониторинг APY

```bash
# Запускать каждый день для сбора данных
dfx canister call yieldex-ic-wallet-manager-backend admin_trigger_apy_collection

# Получить историю за последние 24 часа (при интервале 15 минут = 96 записей)
dfx canister call yieldex-ic-wallet-manager-backend admin_get_apy_history '("AAVE", "USDC", 42161, opt 96)'
```

### Сценарий 3: Анализ ребалансировок

```bash
# Получить последние 10 ребалансировок
dfx canister call yieldex-ic-wallet-manager-backend admin_get_rebalance_history '(opt 10)'

# Получить ребалансировки конкретного пользователя
dfx canister call yieldex-ic-wallet-manager-backend admin_get_user_rebalance_history '(
  principal "hfugy-ahqdz-5sbki-vky4l-xceci-3se5z-2cb7k-jxjuq-qidax-gd53f-nqe",
  opt 20
)'
```

## 🔗 Связанные файлы

- **Модуль:** [src/yieldex-ic-wallet-manager-backend/src/services/apy_parser.rs](src/yieldex-ic-wallet-manager-backend/src/services/apy_parser.rs)
- **Типы:** [src/yieldex-ic-wallet-manager-backend/src/types/scheduler.rs](src/yieldex-ic-wallet-manager-backend/src/types/scheduler.rs)
- **API:** [src/yieldex-ic-wallet-manager-backend/src/lib.rs](src/yieldex-ic-wallet-manager-backend/src/lib.rs) (строки 1258-1449)
- **Интеграция:** [src/yieldex-ic-wallet-manager-backend/src/services/scheduler.rs](src/yieldex-ic-wallet-manager-backend/src/services/scheduler.rs)

---

**Версия:** 1.0.0
**Дата:** 2025-01-21
**Статус:** ✅ Production Ready
