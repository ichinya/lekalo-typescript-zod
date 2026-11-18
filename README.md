# lekalo-typescript-zod

Owns the zod implementation and its regression fixtures, extracted from `lekalo-typescript`. Lekalo core does not import this package.

The `src/` modules are runtime integrations, consumed through explicit package exports. They require only the context passed by the invoking runtime; they do not import its private source tree. These integrations operate on the existing target-protocol IR/compiler API and are distinct from language-neutral contract checks.

Run package tests with `node --test adapters/node-typescript/test/zod-*.test.mjs`. Compiler/host integration tests require `LEKALO_TYPESCRIPT_ROOT` pointing to a built TypeScript runtime checkout. No application code or package lifecycle scripts are executed.

The TypeScript runtime's release build pins this package and includes the selected integration in its distribution. That distribution belongs to the runtime repository, not Lekalo core. Adding another language runtime requires no change to the core.

The root Git rules package requires at least one exported declaration under `validation.schemas/v1`. The TypeScript binding recognizes direct exported const declarations using named imports from `zod`, including aliases. The check covers declaration presence.
