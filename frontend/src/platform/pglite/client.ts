import { PGlite } from "@electric-sql/pglite";
import { live, type PGliteWithLive } from "@electric-sql/pglite/live";
import type { ContentBlock, RichMessage } from "@/types/chat-content";
import type { Chunk } from "@/shared/content/chunk";
import { decodePersistedChatContent } from "@/platform/pglite/legacy-chat-content";
import type { LocalThread } from "@/types";
import {
  loadPgliteFsBundle,
  loadPgliteSeedForFreshDatabase,
  loadPgliteWasmModule,
} from "@/platform/pglite/assets";
import { MIGRATIONS } from "@/platform/pglite/migrations";
import {
  RunEventRepository,
  type AppendPersistedRunEventInput,
  type FinishPersistedRunInput,
  type PersistedRun,
  type PersistedRunEvent,
  type PersistedRunSnapshot,
  type PersistedRunSnapshotSubscription,
  type StartPersistedRunInput,
} from "@/platform/pglite/run-event-repository";

// ---------------------------------------------------------------------------
// Status callback type — emitted during open() for loading-screen feedback
// ---------------------------------------------------------------------------

export type OnStatusFn = (msg: string) => void;

type UarPGlite = PGlite & PGliteWithLive;

// ---------------------------------------------------------------------------
// Module-level singleton — set once by DbProvider, consumed by stores
// ---------------------------------------------------------------------------

let _instance: UarDb | null = null;
let _openPromise: Promise<UarDb> | null = null;

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
  chunks: Chunk[] | string;
  created_at: string;
  status: string;
}

// ---------------------------------------------------------------------------
// UarDb — thin typed wrapper around PGlite
// ---------------------------------------------------------------------------

export class UarDb {
  private readonly runEvents: RunEventRepository;

  private constructor(private readonly db: UarPGlite) {
    this.runEvents = new RunEventRepository(db);
  }

  // ---- lifecycle ----------------------------------------------------------

  static async open(onStatus?: OnStatusFn): Promise<UarDb> {
    if (_instance) return _instance;
    if (_openPromise) {
      onStatus?.("Waiting for local database…");
      return _openPromise;
    }

    onStatus?.("Opening local database…");
    _openPromise = (async () => {
      const [fsBundle, loadDataDir, pgliteWasmModule] = await Promise.all([
        loadPgliteFsBundle(),
        loadPgliteSeedForFreshDatabase(),
        loadPgliteWasmModule(),
      ]);
      const db = await PGlite.create("idb://uar-threads", {
        fsBundle,
        loadDataDir,
        pgliteWasmModule,
        extensions: { live },
      });
      const instance = new UarDb(db);
      if (loadDataDir) {
        onStatus?.("Versioned schema seed loaded");
      } else {
        await instance.runMigrations(onStatus);
      }
      await instance.migrateFromLocalStorage(onStatus);
      onStatus?.("Database ready");
      _instance = instance;
      return instance;
    })().catch((err: unknown) => {
      _openPromise = null;
      throw err;
    });

    return _openPromise;
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
      "SELECT id, thread_id, role, content, chunks, created_at, status FROM messages WHERE thread_id = $1 ORDER BY created_at ASC",
      [threadId],
    );
    return rows.map(rowToMessage);
  }

  async insertMessage(threadId: string, msg: RichMessage): Promise<void> {
    await this.db.query(
      `INSERT INTO messages (id, thread_id, role, content, chunks, created_at, status)
       VALUES ($1, $2, $3, $4, $5, $6, $7)
       ON CONFLICT (id) DO UPDATE
         SET content = EXCLUDED.content,
             chunks  = EXCLUDED.chunks,
             status  = EXCLUDED.status`,
      [
        msg.id,
        threadId,
        msg.role,
        JSON.stringify(msg.content),
        JSON.stringify(msg.chunks ?? []),
        msg.createdAt instanceof Date ? msg.createdAt.toISOString() : msg.createdAt,
        msg.status ?? "complete",
      ],
    );
  }

  // ---- runs and normalized events ---------------------------------------

  /** Platform-only handle used by PEM's PGlite persistence adapter. */
  getPersistenceClient(): PGlite {
    return this.db;
  }

  startRun(input: StartPersistedRunInput): Promise<void> {
    return this.runEvents.startRun(input);
  }

  finishRun(input: FinishPersistedRunInput): Promise<void> {
    return this.runEvents.finishRun(input);
  }

  appendRunEvent(input: AppendPersistedRunEventInput): Promise<number | null> {
    return this.runEvents.appendEvent(input);
  }

  getRuns(): Promise<PersistedRun[]> {
    return this.runEvents.getRuns();
  }

  getRunEvents(runId: string): Promise<PersistedRunEvent[]> {
    return this.runEvents.getRunEvents(runId);
  }

  subscribeRunSnapshot(
    runId: string,
    onSnapshot: (snapshot: PersistedRunSnapshot) => void,
  ): Promise<PersistedRunSnapshotSubscription> {
    return this.runEvents.subscribeRunSnapshot(runId, onSnapshot);
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
  let rawContent: unknown = row.content;
  if (typeof rawContent === "string") {
    try { rawContent = JSON.parse(rawContent) as unknown; }
    catch { rawContent = []; }
  }
  let storedChunks: Chunk[] = [];
  if (typeof row.chunks === "string") {
    try { storedChunks = JSON.parse(row.chunks) as Chunk[]; }
    catch { storedChunks = []; }
  } else if (Array.isArray(row.chunks)) storedChunks = row.chunks;
  const decoded = decodePersistedChatContent(rawContent, {
    messageId: row.id,
    at: row.created_at,
    finalized: row.status !== "in_progress",
  });
  return {
    id: row.id,
    role: row.role,
    content: decoded.content,
    chunks: storedChunks.length > 0 ? storedChunks : decoded.chunks,
    createdAt: new Date(row.created_at),
    status: (row.status as RichMessage["status"]) ?? "complete",
  };
}
