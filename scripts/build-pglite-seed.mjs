#!/usr/bin/env node

import { PGlite } from "../frontend/node_modules/@electric-sql/pglite/dist/index.js";
import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { promisify } from "node:util";

const repositoryRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const execFileAsync = promisify(execFile);
const migrationsEntry = join(
  repositoryRoot,
  "frontend/src/platform/pglite/migrations.ts",
);
const temporaryDirectory = await mkdtemp(join(tmpdir(), "uar-pglite-seed-"));

const initializeSchema = async (database, migrations, migrationDigest) => {
  await database.exec(`
    CREATE TABLE schema_migrations (
      version    INTEGER     PRIMARY KEY,
      applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    )
  `);
  await database.exec(`
    CREATE TABLE _uar_seed_metadata (
      key   TEXT PRIMARY KEY,
      value TEXT NOT NULL
    )
  `);
  await database.query(
    "INSERT INTO _uar_seed_metadata (key, value) VALUES ('migration_sha256', $1)",
    [migrationDigest],
  );
  for (const migration of migrations) {
    await database.exec(migration.up);
    await database.query(
      "INSERT INTO schema_migrations (version, applied_at) VALUES ($1, '2000-01-01T00:00:00Z')",
      [migration.version],
    );
  }
};

const readPublicSchemaCatalog = async (database) => {
  const [tables, columns, constraints, indexes] = await Promise.all([
    database.query(`
      SELECT tablename
      FROM pg_tables
      WHERE schemaname = 'public'
      ORDER BY tablename
    `),
    database.query(`
      SELECT
        table_name,
        column_name,
        ordinal_position::integer AS ordinal_position,
        udt_schema,
        udt_name,
        is_nullable,
        column_default
      FROM information_schema.columns
      WHERE table_schema = 'public'
      ORDER BY table_name, ordinal_position
    `),
    database.query(`
      SELECT
        relation.relname AS table_name,
        constraint_record.conname AS constraint_name,
        constraint_record.contype AS constraint_type,
        pg_get_constraintdef(constraint_record.oid, true) AS definition
      FROM pg_constraint AS constraint_record
      JOIN pg_class AS relation ON relation.oid = constraint_record.conrelid
      JOIN pg_namespace AS namespace ON namespace.oid = relation.relnamespace
      WHERE namespace.nspname = 'public'
      ORDER BY relation.relname, constraint_record.conname
    `),
    database.query(`
      SELECT tablename AS table_name, indexname AS index_name, indexdef AS definition
      FROM pg_indexes
      WHERE schemaname = 'public'
      ORDER BY tablename, indexname
    `),
  ]);
  return {
    tables: tables.rows,
    columns: columns.rows,
    constraints: constraints.rows,
    indexes: indexes.rows,
  };
};

const digestCatalog = (catalog) => createHash("sha256")
  .update(JSON.stringify(catalog))
  .digest("hex");

try {
  const compiledMigrations = join(temporaryDirectory, "migrations.mjs");
  await execFileAsync(
    join(repositoryRoot, "frontend/node_modules/.bin/esbuild"),
    [
      migrationsEntry,
      "--bundle",
      "--format=esm",
      "--platform=node",
      `--outfile=${compiledMigrations}`,
      "--log-level=error",
    ],
  );

  const { MIGRATIONS } = await import(pathToFileURL(compiledMigrations));
  const expectedVersions = MIGRATIONS.map(({ version }) => version);
  const migrationDigest = createHash("sha256")
    .update(JSON.stringify(MIGRATIONS.map(({ version, name, up }) => ({ version, name, up }))))
    .digest("hex");
  const latestVersion = expectedVersions.at(-1);
  const seedPath = join(
    repositoryRoot,
    `frontend/src/platform/pglite/pglite-seed-v${latestVersion}.tar.gz`,
  );

  if (process.argv.includes("--check")) {
    const seed = await readFile(seedPath);
    const database = await PGlite.create({ loadDataDir: new Blob([seed]) });
    const referenceDatabase = await PGlite.create();
    try {
      await initializeSchema(referenceDatabase, MIGRATIONS, migrationDigest);
      const { rows } = await database.query(
        "SELECT version FROM schema_migrations ORDER BY version",
      );
      const actualVersions = rows.map(({ version }) => version);
      if (JSON.stringify(actualVersions) !== JSON.stringify(expectedVersions)) {
        throw new Error(
          `PGlite seed migration mismatch: expected ${expectedVersions.join(",")}, received ${actualVersions.join(",")}`,
        );
      }

      const metadata = await database.query(
        "SELECT value FROM _uar_seed_metadata WHERE key = 'migration_sha256'",
      );
      if (metadata.rows[0]?.value !== migrationDigest) {
        throw new Error(
          `PGlite seed migration digest mismatch: expected ${migrationDigest}, received ${metadata.rows[0]?.value ?? "<missing>"}`,
        );
      }

      const [actualCatalog, expectedCatalog] = await Promise.all([
        readPublicSchemaCatalog(database),
        readPublicSchemaCatalog(referenceDatabase),
      ]);
      const actualCatalogDigest = digestCatalog(actualCatalog);
      const expectedCatalogDigest = digestCatalog(expectedCatalog);
      if (actualCatalogDigest !== expectedCatalogDigest) {
        throw new Error(
          `PGlite seed schema catalog mismatch: expected ${expectedCatalogDigest}, received ${actualCatalogDigest}`,
        );
      }

      const metadataTables = new Set(["schema_migrations", "_uar_seed_metadata"]);
      for (const { tablename } of actualCatalog.tables) {
        if (metadataTables.has(tablename)) continue;
        const quotedTable = `"${tablename.replaceAll('"', '""')}"`;
        const count = await database.query(`SELECT COUNT(*)::integer AS count FROM ${quotedTable}`);
        if (count.rows[0]?.count !== 0) {
          throw new Error(`PGlite seed contains product data in ${tablename}`);
        }
      }
      console.log(
        JSON.stringify({
          status: "valid",
          path: seedPath,
          migrations: expectedVersions,
          migrationSha256: migrationDigest,
          schemaCatalogSha256: actualCatalogDigest,
          bytes: seed.byteLength,
        }),
      );
    } finally {
      await database.close();
      await referenceDatabase.close();
    }
  } else {
    const database = await PGlite.create();
    try {
      await initializeSchema(database, MIGRATIONS, migrationDigest);

      const archive = await database.dumpDataDir("gzip");
      const bytes = new Uint8Array(await archive.arrayBuffer());
      await writeFile(seedPath, bytes);
      console.log(
        JSON.stringify({
          status: "built",
          path: seedPath,
          migrations: expectedVersions,
          migrationSha256: migrationDigest,
          bytes: bytes.byteLength,
        }),
      );
    } finally {
      await database.close();
    }
  }
} finally {
  await rm(temporaryDirectory, { recursive: true, force: true });
}
