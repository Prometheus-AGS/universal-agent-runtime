import { PGlite } from "@electric-sql/pglite";
import type { ContentBlock, RichMessage } from "@/types/chat-content";
import type { LocalThread } from "@/types";

// ---------------------------------------------------------------------------
// Status callback type — emitted during open() for loading-screen feedback
// ---------------------------------------------------------------------------

export type OnStatusFn = (msg: string) => void;

// ---------------------------------------------------------------------------
// Module-level singleton — set once by DbProvider, consumed by stores
// ---------------------------------------------------------------------------

let _instance: UarDb | null = null;

export function setDbInstance(db: UarDb): void {
  _instance = db;
}

/** Returns the initialized UarDb. Throws if called before DbProvider is ready. */
export function getDbInstance(): UarDb {
  if (!_instance) throw new Error("[UarDb] Database not yet initialized");
  return _instance;
}

// ---------------------------------------------------------------------------
// Schema migrations
// ---------------------------------------------------------------------------

interface Migration {
  version: number;
  name: string;
  up: string;
}

const MIGRATIONS: Migration[] = [
  {
    version: 1,
    name: "initial_schema",
    up: `
      CREATE TABLE IF NOT EXISTS threads (
        id          TEXT        PRIMARY KEY,
        title       TEXT        NOT NULL DEFAULT 'New conversation',
        is_ephemeral BOOLEAN    NOT NULL DEFAULT TRUE,
        created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
      );

      CREATE TABLE IF NOT EXISTS messages (
        id         TEXT        PRIMARY KEY,
        thread_id  TEXT        NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
        role       TEXT        NOT NULL CHECK (role IN ('user','assistant','system')),
        content    JSONB       NOT NULL DEFAULT '[]',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        status     TEXT        NOT NULL DEFAULT 'complete'
      );

      CREATE INDEX IF NOT EXISTS idx_messages_thread_id  ON messages(thread_id);
      CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(thread_id, created_at);
    `,
  },
];

// ---------------------------------------------------------------------------
// Row shapes returned from PGlite queries
// ---------------------------------------------------------------------------

interface ThreadRow {
  id: string;
  title: string;
  is_ephemeral: boolean;
  created_at: string;
  updated_at: string;
}

interface MessageRow {
  id: string;
  thread_id: string;
  role: "user" | "assistant" | "system";
  content: ContentBlock[] | string; // PGlite may return JSONB already parsed or as string
  created_at: string;
  status: string;
}

// ---------------------------------------------------------------------------
// UarDb — thin typed wrapper around PGlite
// ---------------------------------------------------------------------------

export class UarDb {
  private constructor(private readonly db: PGlite) {}

  // ---- lifecycle ----------------------------------------------------------

  static async open(onStatus?: OnStatusFn): Promise<UarDb> {
    onStatus?.("Opening local database…");
    const db = new PGlite("idb://uar-threads");
    const instance = new UarDb(db);
    await instance.runMigrations(onStatus);
    await instance.migrateFromLocalStorage(onStatus);
    onStatus?.("Database ready");
    return instance;
  }

  private async runMigrations(onStatus?: OnStatusFn): Promise<void> {
    onStatus?.("Bootstrapping schema migrations table…");
    await this.db.exec(`
      CREATE TABLE IF NOT EXISTS schema_migrations (
        version    INTEGER     PRIMARY KEY,
        applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
      );
    `);

    for (const m of MIGRATIONS) {
      const { rows } = await this.db.query<{ version: number }>(
        "SELECT version FROM schema_migrations WHERE version = $1",
        [m.version],
      );
      if (rows.length === 0) {
        onStatus?.(`Applying migration ${m.version}: ${m.name}…`);
        await this.db.exec(m.up);
        await this.db.query(
          "INSERT INTO schema_migrations (version) VALUES ($1)",
          [m.version],
        );
        onStatus?.(`Migration ${m.version} applied`);
      } else {
        onStatus?.(`Migration ${m.version} (${m.name}) already applied`);
      }
    }
  }

  /** One-time migration of any data that was previously stored in localStorage. */
  private async migrateFromLocalStorage(onStatus?: OnStatusFn): Promise<void> {
    const flag = "uar-pglite-migrated-v1";
    if (localStorage.getItem(flag)) {
      onStatus?.("No legacy localStorage data to migrate");
      return;
    }

    onStatus?.("Migrating legacy data from localStorage…");

    // Migrate thread registry
    const rawRegistry = localStorage.getItem("uar-thread-registry");
    if (rawRegistry) {
      try {
        const parsed = JSON.parse(rawRegistry) as { state?: { threads?: Record<string, LocalThread> } };
        const threads = parsed.state?.threads ?? {};
        const count = Object.keys(threads).length;
        onStatus?.(`Migrating ${count} legacy thread(s)…`);
        for (const t of Object.values(threads)) {
          await this.upsertThread(t).catch(() => { /* skip invalid rows */ });
        }
        onStatus?.(`Migrated ${count} thread(s)`);
      } catch { /* ignore parse errors */ }
    } else {
      onStatus?.("No legacy thread registry found");
    }

    // Migrate message store
    const rawMessages = localStorage.getItem("uar-chat-messages");
    if (rawMessages) {
      try {
        const parsed = JSON.parse(rawMessages) as { state?: { messagesByThread?: Record<string, RichMessage[]> } };
        const byThread = parsed.state?.messagesByThread ?? {};
        const totalMsgs = Object.values(byThread).reduce((n, msgs) => n + msgs.length, 0);
        onStatus?.(`Migrating ${totalMsgs} legacy message(s)…`);
        for (const [threadId, msgs] of Object.entries(byThread)) {
          // Ensure thread exists before inserting messages
          const exists = await this.db.query<{ id: string }>(
            "SELECT id FROM threads WHERE id = $1",
            [threadId],
          );
          if (exists.rows.length === 0) continue;
          for (const msg of msgs) {
            await this.insertMessage(threadId, msg).catch(() => { /* skip duplicates */ });
          }
        }
        onStatus?.(`Migrated ${totalMsgs} message(s)`);
      } catch { /* ignore parse errors */ }
    } else {
      onStatus?.("No legacy messages found");
    }

    // Clean up localStorage keys that are now superseded
    localStorage.removeItem("uar-thread-registry");
    localStorage.removeItem("uar-chat-messages");
    localStorage.setItem(flag, "1");
    onStatus?.("Legacy migration complete — localStorage cleared");
  }

  // ---- threads ------------------------------------------------------------

  async getThreads(): Promise<LocalThread[]> {
    const { rows } = await this.db.query<ThreadRow>(
      "SELECT id, title, is_ephemeral, created_at, updated_at FROM threads ORDER BY updated_at DESC",
    );
    return rows.map(rowToThread);
  }

  async upsertThread(thread: LocalThread): Promise<void> {
    await this.db.query(
      `INSERT INTO threads (id, title, is_ephemeral, created_at, updated_at)
       VALUES ($1, $2, $3, $4, $5)
       ON CONFLICT (id) DO UPDATE
         SET title = EXCLUDED.title,
             is_ephemeral = EXCLUDED.is_ephemeral,
             updated_at = EXCLUDED.updated_at`,
      [thread.id, thread.title, thread.isEphemeral, thread.createdAt, thread.updatedAt],
    );
  }

  async deleteThread(id: string): Promise<void> {
    await this.db.query("DELETE FROM threads WHERE id = $1", [id]);
  }

  async touchThread(id: string): Promise<void> {
    const now = new Date().toISOString();
    await this.db.query(
      "UPDATE threads SET updated_at = $1 WHERE id = $2",
      [now, id],
    );
  }

  // ---- messages -----------------------------------------------------------

  async getMessages(threadId: string): Promise<RichMessage[]> {
    const { rows } = await this.db.query<MessageRow>(
      "SELECT id, thread_id, role, content, created_at, status FROM messages WHERE thread_id = $1 ORDER BY created_at ASC",
      [threadId],
    );
    return rows.map(rowToMessage);
  }

  async insertMessage(threadId: string, msg: RichMessage): Promise<void> {
    await this.db.query(
      `INSERT INTO messages (id, thread_id, role, content, created_at, status)
       VALUES ($1, $2, $3, $4, $5, $6)
       ON CONFLICT (id) DO UPDATE
         SET content = EXCLUDED.content,
             status  = EXCLUDED.status`,
      [
        msg.id,
        threadId,
        msg.role,
        JSON.stringify(msg.content),
        msg.createdAt instanceof Date ? msg.createdAt.toISOString() : msg.createdAt,
        msg.status ?? "complete",
      ],
    );
  }
}

// ---------------------------------------------------------------------------
// Row → domain type converters
// ---------------------------------------------------------------------------

function rowToThread(row: ThreadRow): LocalThread {
  return {
    id: row.id,
    title: row.title,
    isEphemeral: row.is_ephemeral,
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}

function rowToMessage(row: MessageRow): RichMessage {
  let content: ContentBlock[];
  if (typeof row.content === "string") {
    try { content = JSON.parse(row.content) as ContentBlock[]; }
    catch { content = []; }
  } else {
    content = row.content as ContentBlock[];
  }
  return {
    id: row.id,
    role: row.role,
    content,
    createdAt: new Date(row.created_at),
    status: (row.status as RichMessage["status"]) ?? "complete",
  };
}
