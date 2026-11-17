# lekalo-typescript-zod

Пакет правил и генерации Zod для [lekalo-typescript](https://github.com/ichinya/lekalo-typescript).

## Статус

Репозиторий подготовлен для выделения существующей реализации. Код ещё не перенесён; пакет пока не готов к установке или использованию.

- Перенос реализации: [#1](https://github.com/ichinya/lekalo-typescript-zod/issues/1).
- Общий план: [lekalo#241](https://github.com/ichinya/lekalo/issues/241).

## Ответственность пакета

- Отображение нейтральных типов в поддерживаемые Zod-схемы.
- Генерация и проверка Zod artifacts, policy и ограничения поддерживаемого subset.
- Zod-specific generated runtime, fixtures и dependency/version pins.

## Источник переноса

`lekalo-typescript/adapters/node-typescript/src/zod-{map,emit,gen,policy}.mjs`, `adapter-zod.mjs`, Zod-часть `generation-composite.mjs` и связанные tests/fixtures.

## Границы

Canonical Model/IR и host-права на запись остаются в ядре. Общий runtime только компонует выбранные генераторы; порядок, конфликты и unsupported случаи явные.

Версия пакета, версия provider/protocol API и версия языка или движка приложения фиксируются отдельно. Сборка и тесты должны быть автономными; источники и зависимости закрепляются точными версиями и digest. Выбор произвольного совместимого Git/local package и загрузка его зависимостей отслеживаются в [lekalo#239](https://github.com/ichinya/lekalo/issues/239).
