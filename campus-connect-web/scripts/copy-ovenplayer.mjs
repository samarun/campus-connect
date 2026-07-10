import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const source = join(root, "node_modules", "ovenplayer", "dist", "ovenplayer.js");
const target = join(root, "public", "vendor", "ovenplayer", "ovenplayer.js");

if (!existsSync(source)) {
  console.warn("OvenPlayer distribution file was not found; install dependencies first.");
  process.exit(0);
}

mkdirSync(dirname(target), { recursive: true });
copyFileSync(source, target);
console.log("Copied OvenPlayer to public/vendor/ovenplayer/ovenplayer.js");

