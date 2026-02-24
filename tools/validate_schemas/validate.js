"use strict";

const Ajv = require("ajv/dist/2020");
const addFormats = require("ajv-formats");
const fs = require("fs");
const path = require("path");

const schemasDir = process.env.SCHEMAS_DIR;
if (!schemasDir) {
  console.error("SCHEMAS_DIR environment variable is not set");
  process.exit(1);
}

if (!fs.existsSync(schemasDir)) {
  console.error("schemas directory not found: " + schemasDir);
  process.exit(1);
}

const files = fs.readdirSync(schemasDir).filter(function (f) {
  return f.endsWith(".schema.json");
});

if (files.length === 0) {
  console.error("No *.schema.json files found in " + schemasDir);
  process.exit(1);
}

const ajv = new Ajv({ strict: true });
addFormats(ajv);

const schemas = files.map(function (f) {
  return JSON.parse(fs.readFileSync(path.join(schemasDir, f), "utf8"));
});

// Add all schemas so cross-file $ref resolution works
schemas.forEach(function (s) {
  ajv.addSchema(s);
});

// Compile each schema
schemas.forEach(function (s) {
  ajv.compile(s);
});

console.log("All schemas compiled successfully: " + files.length);
