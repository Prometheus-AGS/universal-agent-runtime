import type { BrowserContext, Page, Request, Route } from '@playwright/test';
import { purgeBddAgents, restoreS1Baseline } from './api';
import { releaseAllStubs, resetAllStubs } from './stubs';

/**
 * Per-scenario state of the S1 live-model-choice features (phase
 * uar-uiux-architecture-2026-11). It holds what a step records for a later step:
 * the harness-clock windows around each system-default change, the browser's
 * writes, the reply text captured at send time, the conversations the harness
 * created, and the held live-route acknowledgement.
 *
 * Nothing here runs until `begin()`, which only the S1 Background step calls, so
 * the shared fixture costs the other features nothing.
 */

/** Harness-clock window (ms since epoch) around one system-default change. */
export interface ChangeWindow {
  /** Provider-qualified pair the default became, e.g. `openai/gpt-5.4-mini`. */
  pair: string;
  t0: number;
  t1: number;
}

/** A write the browser sent to `/api/uar/**` (GET, HEAD and OPTIONS excluded). */
export interface BrowserWrite {
  method: string;
  path: string;
  body: string | null;
  at: number;
}

/**
 * A conversation the harness created. A thread is hidden from the sidebar until
 * its first message is sent, so a conversation is only a harness handle until
 * then: its override and agent are recorded and re-applied when the handle has
 * to be recreated on a fresh thread, and it is renamed after its first message.
 */
export interface Conversation {
  name: string;
  agent: string | null;
  /** Provider-qualified override the harness set, or null. */
  override: string | null;
  /** True once the thread has been renamed to `name` and appears in the sidebar. */
  materialized: boolean;
}

/** The held `POST /api/uar/providers/<id>/default` of the route-hold steps. */
export interface HeldAcknowledgement {
  providerId: string;
  /** Provider-qualified pair being confirmed. */
  pair: string;
  /** Harness clock just before the confirm click. */
  t0: number;
  /** The intercepted request's route, once the handler has seen it. */
  route: Route | null;
  /** Status of the PUT that precedes the live route, once the response listener saw it. */
  putStatus: number | null;
  /** After release or failure, later matching requests pass through to the server. */
  passThrough: boolean;
  /** Harness clock when the held request was fulfilled with a failure status. */
  failedAt: number | null;
  /** Resolves when the handler has intercepted the exact live-route POST. */
  intercepted: Promise<void>;
}

const READ_METHODS = new Set(['GET', 'HEAD', 'OPTIONS']);

export class S1Context {
  active = false;
  readonly windows: ChangeWindow[] = [];
  readonly writes: BrowserWrite[] = [];
  /** Reply text (`bdd-reply-...`) the transcript showed after each sent message. */
  readonly replyShown = new Map<string, string>();
  /** How many `bdd-reply-...` strings the transcript held when a message was sent. */
  readonly replyCountBefore = new Map<string, number>();
  readonly conversations = new Map<string, Conversation>();
  currentConversation: string | null = null;
  /** Text a refusal produced: visible errors and the response bodies of the refused request. */
  refusalTexts: string[] = [];
  held: HeldAcknowledgement | null = null;
  /** Index into `writes` where the latest operator change started. */
  changeWritesFrom = 0;
  /** Index into `writes` where the latest operator change ended. */
  changeWritesTo = 0;
  /** Latest session id seen in a `/api/uar/sessions/<id>/` request. */
  lastSessionId: string | null = null;
  private operator: Page | null = null;
  private requestListener: ((request: Request) => void) | null = null;

  constructor(
    readonly page: Page,
    readonly context: BrowserContext,
  ) {}

  /**
   * Starts the scenario: clears every stub's request log, removes the other
   * features' `BDD Agent *` residue, and starts recording the browser's writes.
   */
  async begin(): Promise<void> {
    if (this.active) return;
    this.active = true;
    await resetAllStubs();
    await purgeBddAgents();
    this.requestListener = (request: Request) => {
      const { pathname } = new URL(request.url());
      const session = /^\/api\/uar\/sessions\/([^/]+)\//.exec(pathname);
      if (session) this.lastSessionId = decodeURIComponent(session[1]);
      if (!pathname.startsWith('/api/uar/') || READ_METHODS.has(request.method())) return;
      this.writes.push({
        method: request.method(),
        path: pathname,
        body: request.postData(),
        at: Date.now(),
      });
    };
    this.context.on('request', this.requestListener);
  }

  /**
   * Ends the scenario: frees any held reply so a failed scenario does not leave a
   * request parked, then puts the shared server's providers and default back.
   */
  async dispose(): Promise<void> {
    if (!this.active) return;
    if (this.requestListener) this.context.off('request', this.requestListener);
    if (this.held?.route && !this.held.passThrough) {
      this.held.passThrough = true;
      await this.held.route.abort().catch(() => undefined);
    }
    await releaseAllStubs({ allowMissingRoute: true });
    await restoreS1Baseline();
    if (this.operator && !this.operator.isClosed()) await this.operator.close();
    this.active = false;
  }

  /**
   * The page the operator acts on: a second page of the same browser context,
   * so the chat page keeps its conversation while the console changes the
   * default (this is also the "from another page" of the in-flight scenario).
   */
  async operatorPage(): Promise<Page> {
    if (!this.operator || this.operator.isClosed()) this.operator = await this.context.newPage();
    return this.operator;
  }

  recordWindow(pair: string, t0: number, t1: number): void {
    this.windows.push({ pair, t0, t1 });
  }

  /** The most recent window recorded for a pair, or undefined. */
  latestWindowFor(pair: string): ChangeWindow | undefined {
    return this.windows.filter((window) => window.pair === pair).at(-1);
  }

  /** Time (ms) the latest default change finished, or undefined before any. */
  latestChangeEnd(): number | undefined {
    return this.windows.at(-1)?.t1;
  }

  /** Writes recorded between two indexes of the browser's write log. */
  writesBetween(from: number, to: number): BrowserWrite[] {
    return this.writes.slice(from, to);
  }

  currentConversationState(): Conversation | null {
    return this.currentConversation ? (this.conversations.get(this.currentConversation) ?? null) : null;
  }
}

// ---------------------------------------------------------------------------
// Receipt time comparator (`fixtures/s1-oracle.json` `timeComparator`): second
// precision, shown `HH:MM:SS` in the browser's time zone. The browser and the
// harness share one host, so the harness clock and zone are the truth.
// ---------------------------------------------------------------------------

export const TIME_TOLERANCE_MS = 2_000;

/** Seconds since local midnight of the first `HH:MM:SS` in `text`, or null. */
export function parseClock(text: string): number | null {
  const match = /(\d{2}):(\d{2}):(\d{2})/.exec(text);
  if (!match) return null;
  return Number(match[1]) * 3600 + Number(match[2]) * 60 + Number(match[3]);
}

/**
 * Whether a displayed clock time lies inside a change window widened by the
 * 2 second tolerance. The clock carries no date, so the window's own local day
 * and its neighbours are tried.
 */
export function clockInsideWindow(clockSeconds: number, window: ChangeWindow): boolean {
  const day = new Date(window.t1);
  const lower = Math.floor(window.t0 / 1000) * 1000 - TIME_TOLERANCE_MS;
  const upper = Math.floor(window.t1 / 1000) * 1000 + TIME_TOLERANCE_MS;
  const hours = Math.floor(clockSeconds / 3600);
  const minutes = Math.floor((clockSeconds % 3600) / 60);
  const seconds = clockSeconds % 60;
  return [-1, 0, 1].some((shift) => {
    const candidate = new Date(day.getFullYear(), day.getMonth(), day.getDate() + shift, hours, minutes, seconds).getTime();
    return candidate >= lower && candidate <= upper;
  });
}
