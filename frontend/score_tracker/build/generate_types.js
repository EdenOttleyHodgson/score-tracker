import * as fs from "fs/promises";
import * as path from "path";
import * as url from "url";
import { compile } from "json-schema-to-typescript";

const dirname = path.join(
  url.fileURLToPath(import.meta.url),
  "..",
  "..",
  "..",
  "..",
  "backend",
  "type_schemas"
);
const out_dirname = path.join(
  url.fileURLToPath(import.meta.url),
  "..",
  "..",
  "app",
  "backend"
);
async function main() {
  let schema_files = (await fs.readdir(dirname)).filter((x) =>
    x.endsWith(".json")
  );
  let types = new Set();
  for (let filename of schema_files) {
    let f_path = path.join(dirname, filename);
    let schema = JSON.parse(await fs.readFile(f_path));
    let compiled = await compile(schema, schema.title, {
      bannerComment: "",
      inferStringEnumKeysFromValues: true,
    });
    for (let type of compiled.split("export")) {
      if (!type) {
        continue;
      }
      types.add("export " + type.trim());
    }
  }
  let output = Array.from(types).join("\n\n");
  let output_path = path.join(out_dirname, "types.ts");
  try {
    let existing = await fs.readFile(output_path);
    if (existing == output) {
      return;
    }
  } catch (e) {
    if (e.code !== "ENOENT") {
      throw e;
    }
  }
  await fs.writeFile(output_path, output);
}
main().catch((e) => {
  console.error(e);
  process.exit(1);
});
