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
  getRealtimeManager,
  getSchema,
  registerEntityFromSql,
  registerSchema,
  serializeKey,
  startLocalFirstGraph,
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
  EntityType,
  FilterClause,
  GraphPersistenceAdapter,
  LocalFirstGraphRuntime,
  PGlitePersistenceClient,
  RealtimeAdapter,
  ReplayRetryPolicy,
  SubscriptionConfig,
  UnsubscribeFn,
  ViewDescriptor,
} from "@prometheus-ags/prometheus-entity-management";
