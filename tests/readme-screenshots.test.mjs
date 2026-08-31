import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { readdir, stat } from "node:fs/promises";
import { join } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const readme = await readFile(join(root, "README.md"), "utf8");
const references = [...readme.matchAll(/\]\((docs\/screenshots\/[^)]+\.png)\)/g)].map((match) => match[1]);
const expected = ["docs/screenshots/login.png", "docs/screenshots/operator-dashboard.png", "docs/screenshots/receiver-grid.png"];
assert.deepEqual(references.sort(), expected.sort(), "README must reference exactly the baseline screenshots");

const screenshotDir = join(root, "docs/screenshots");
const files = (await readdir(screenshotDir)).filter((file) => file.endsWith(".png")).map((file) => `docs/screenshots/${file}`);
assert.deepEqual(files.sort(), references.sort(), "every PNG must be referenced by README");

for (const relativePath of files) {
  const bytes = await readFile(join(root, relativePath));
  assert.deepEqual([...bytes.subarray(0, 8)], [137, 80, 78, 71, 13, 10, 26, 10], `${relativePath} is not a PNG`);
  assert.equal(bytes.readUInt32BE(16), 1440, `${relativePath} must be 1440px wide`);
  assert.equal(bytes.readUInt32BE(20), 900, `${relativePath} must be 900px high`);
}
console.log(`Validated ${files.length} README screenshots at 1440x900.`);
