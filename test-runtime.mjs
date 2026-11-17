import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
if (!process.env.LEKALO_TYPESCRIPT_ROOT) throw new Error('Set LEKALO_TYPESCRIPT_ROOT to a built TypeScript runtime for integration tests');
export const runtimeRoot = resolve(process.env.LEKALO_TYPESCRIPT_ROOT);
export const runtimeModule = path => import(pathToFileURL(resolve(runtimeRoot, 'adapters/node-typescript', path)).href);
