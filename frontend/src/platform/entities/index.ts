/**
 * Application-owned entry point for the Prometheus entity graph runtime.
 *
 * Keep package integration centralized here so feature code depends on the
 * platform contract rather than the external package path.
 */
export {
  configureEngine,
  createPGlitePersistenceAdapter,
  getEntityJsonSchema,
  getRegisteredEntityTypes,
  getRealtimeManager,
  getSchema,
  graphStore,
  registerEntityFromSql,
  registerEntityTransport,
  registerSchema,
  serializeKey,
  startLocalFirstGraph,
  useEntities,
  useEntityView,
  useGraphStore,
} from "@prometheus-ags/prometheus-entity-management";

export type {
  AdapterStatus,
  ChangeOperation,
  ChangeSet,
  ChannelConfig,
  EntityChange,
  EntityId,
  EntityTransport,
  EntityType,
  FilterClause,
  GraphPersistenceAdapter,
  ListQuery,
  ListResult,
  LocalFirstGraphRuntime,
  PGlitePersistenceClient,
  RealtimeAdapter,
  ReplayRetryPolicy,
  SubscriptionConfig,
  UnsubscribeFn,
  ViewDescriptor,
} from "@prometheus-ags/prometheus-entity-management";

export * from "./session-configuration";
export * from "./presentations";
export * from "./presentation-assignments";
export * from "./presentation-provenance";
