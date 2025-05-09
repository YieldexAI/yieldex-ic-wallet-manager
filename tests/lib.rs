// Тестирование Rust канистр на Internet Computer
// 
// Для тестирования функции now() и API ic_cdk более надежный подход - 
// использовать dfx deploy и проверять функцию напрямую через командную строку:
//
// dfx start --clean
// dfx deploy
// dfx canister call yieldex-ic-wallet-manager-backend now
//
// PocketIC имеет проблемы совместимости с последними версиями зависимостей,
// поэтому для простого юнит-тестирования лучше использовать прямой вызов через dfx 

// Тестирование Rust канистр на Internet Computer с использованием PocketIC
// 
// Для запуска тестов необходимо:
// 1. Установить PocketIC сервер и экспортировать переменную POCKET_IC_BIN
// 2. Запустить тесты: cargo test -p yieldex-ic-wallet-manager-tests

#[cfg(test)]
pub mod pocket_ic_tests; 