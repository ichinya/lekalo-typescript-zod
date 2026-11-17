#!/usr/bin/env node
import {runExternal} from "./external-tool.mjs";
runExternal("lekalo-typescript-drizzle", "scripts/gen-zod-golden.mjs");
