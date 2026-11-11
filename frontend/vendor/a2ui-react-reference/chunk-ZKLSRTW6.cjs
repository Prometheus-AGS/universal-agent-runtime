'use strict';

var React = require('react');
var v0_9 = require('@a2ui/web_core/v0_9');
var basic_catalog = require('@a2ui/web_core/v0_9/basic_catalog');
var universal = require('@a2ui/web_core/v0_9/universal');
var reactDom = require('react-dom');
var jsxRuntime = require('react/jsx-runtime');

function _interopDefault (e) { return e && e.__esModule ? e : { default: e }; }

var React__default = /*#__PURE__*/_interopDefault(React);

// src/A2uiSurface.tsx

// src/host_registry.ts
var HostOwner = class {
};
var rendered = /* @__PURE__ */ new WeakMap();
function markReactRendered(element, owner) {
  rendered.set(element, owner);
}
function isReactRendered(element) {
  return rendered.has(element);
}
function ownerAbove(element) {
  let current = element.parentNode;
  while (current) {
    const owner = current instanceof Element ? rendered.get(current) : void 0;
    if (owner) return owner;
    current = current instanceof ShadowRoot ? current.host : current.parentNode;
  }
  return null;
}
var HostRegistry = class _HostRegistry {
  static registries = /* @__PURE__ */ new WeakMap();
  /** The registry of `surface`; hosts find it through their node's context. */
  static forSurface(surface) {
    let registry = _HostRegistry.registries.get(surface);
    if (!registry) {
      registry = new _HostRegistry();
      _HostRegistry.registries.set(surface, registry);
    }
    return registry;
  }
  hosts = /* @__PURE__ */ new Map();
  snapshots = /* @__PURE__ */ new Map();
  listeners = /* @__PURE__ */ new Map();
  add(host, node, owner) {
    const current = this.hosts.get(host);
    if (current && current.entry.node === node && current.owner === owner) return;
    this.hosts.set(host, { entry: { host, node }, owner });
    if (current && current.owner !== owner) this.publish(current.owner);
    this.publish(owner);
  }
  delete(host) {
    const current = this.hosts.get(host);
    if (!current) return;
    this.hosts.delete(host);
    this.publish(current.owner);
  }
  has(host) {
    return this.hosts.has(host);
  }
  /** Every registered host, in registration order. */
  entries() {
    return [...this.hosts.values()].map(({ entry }) => entry);
  }
  /** Notifies `listener` when the hosts owned by `owner` change. */
  subscribe(owner, listener) {
    let listeners = this.listeners.get(owner);
    if (!listeners) {
      listeners = /* @__PURE__ */ new Set();
      this.listeners.set(owner, listeners);
    }
    listeners.add(listener);
    return () => {
      listeners.delete(listener);
    };
  }
  /** The hosts owned by `owner`; the same array until they change. */
  getSnapshot(owner) {
    let snapshot = this.snapshots.get(owner);
    if (!snapshot) {
      snapshot = this.compute(owner);
      this.snapshots.set(owner, snapshot);
    }
    return snapshot;
  }
  compute(owner) {
    const result = [];
    for (const { entry, owner: entryOwner } of this.hosts.values()) {
      if (entryOwner === owner) result.push(entry);
    }
    return result;
  }
  publish(owner) {
    this.snapshots.set(owner, this.compute(owner));
    for (const listener of this.listeners.get(owner) ?? []) listener();
  }
};

// src/catalog/react_host_element.ts
var ReactWcHost = class extends HTMLElement {
  context;
  _node;
  registry;
  connectedCallback() {
    this.style.display = "contents";
    this.register();
  }
  disconnectedCallback() {
    queueMicrotask(() => {
      if (!this.isConnected) {
        this.registry?.delete(this);
        this.registry = void 0;
      }
    });
  }
  set node(node) {
    this._node = node;
    this.context = node?.context;
    this.register();
  }
  get node() {
    return this._node;
  }
  register() {
    const node = this._node;
    const surface = node?.context?.dataContext.surface;
    if (!this.isConnected || !node || !surface || isReactRendered(this)) return;
    const registry = HostRegistry.forSurface(surface);
    if (this.registry !== registry) {
      this.registry?.delete(this);
      this.registry = registry;
    }
    registry.add(this, node, ownerAbove(this));
  }
};

// src/catalog/to_web_component.tsx
var hosted = /* @__PURE__ */ new WeakMap();
var tagNameCounts = /* @__PURE__ */ new Map();
function computeTagName(name) {
  const baseTagName = `a2ui-react-${name.toLowerCase()}`;
  const count = (tagNameCounts.get(baseTagName) ?? 0) + 1;
  tagNameCounts.set(baseTagName, count);
  return count === 1 ? baseTagName : `${baseTagName}-${count}`;
}
function toWebComponent(implementation) {
  if (universal.isWebComponentImplementation(implementation)) {
    return implementation;
  }
  const cached = hosted.get(implementation);
  if (cached) {
    return cached;
  }
  const entry = {
    ...implementation,
    tagName: computeTagName(implementation.name),
    // `customElements` refuses to define one class under two tag names.
    element: class extends ReactWcHost {
    }
  };
  hosted.set(implementation, entry);
  return entry;
}

// src/catalog/prepare_catalogs.ts
var prepared = /* @__PURE__ */ new WeakSet();
function prepare(catalog) {
  if (prepared.has(catalog)) return;
  prepared.add(catalog);
  const components = catalog.components;
  for (const [name, implementation] of components) {
    if (!universal.isWebComponentImplementation(implementation)) {
      components.set(name, toWebComponent(implementation));
    }
  }
}
function prepareCatalogs(surface) {
  if (surface.defaultCatalog) prepare(surface.defaultCatalog);
  for (const catalog of surface.availableCatalogs.values()) prepare(catalog);
}
function assign(element, node) {
  element.context = node.context;
  element.node = node;
}
var WebComponentNode = React.memo(
  ({
    node,
    owner,
    children
  }) => {
    const implementation = node.impl;
    if (!universal.isWebComponentImplementation(implementation)) {
      throw new Error(
        `A2UI catalog entry '${node.type}' has no element. A2uiSurface prepares the surface's default and available catalogs; this entry came from another catalog.`
      );
    }
    universal.registerUniversalElement(implementation);
    const elRef = React.useRef(null);
    const nodeRef = React.useRef(node);
    nodeRef.current = node;
    const setRef = React.useCallback(
      (element) => {
        elRef.current = element;
        if (element) {
          if (owner) markReactRendered(element, owner);
          assign(element, nodeRef.current);
        }
      },
      [owner]
    );
    React.useEffect(() => {
      if (elRef.current) {
        assign(elRef.current, node);
      }
    }, [node]);
    return React__default.default.createElement(implementation.tagName, { ref: setRef }, children);
  }
);
WebComponentNode.displayName = "WebComponentNode";
function contextOf(node) {
  if (!node.context) {
    throw new Error(`A2UI views render only resolved nodes; '${node.componentId}' has no context.`);
  }
  return node.context;
}
var LoadingPlaceholder = ({ componentId }) => /* @__PURE__ */ jsxRuntime.jsxs("div", { style: { color: "gray", padding: "4px" }, children: [
  "[Loading ",
  componentId,
  "...]"
] });
var reportedUnresolved = /* @__PURE__ */ new WeakMap();
var UnresolvedChildReference = ({ surface, id, requestedPath, detail }) => {
  const message = `Unresolved child reference '${id}' at '${requestedPath}': ${detail}`;
  React.useEffect(() => {
    let seen = reportedUnresolved.get(surface);
    if (!seen) {
      seen = /* @__PURE__ */ new Set();
      reportedUnresolved.set(surface, seen);
    }
    const key = JSON.stringify([id, requestedPath]);
    if (!seen.has(key)) {
      seen.add(key);
      void surface.dispatchError({ code: "UNRESOLVED_CHILD_REFERENCE", message });
    }
  }, [surface, id, requestedPath, message]);
  return /* @__PURE__ */ jsxRuntime.jsx("div", { style: { color: "red" }, children: message });
};
function useSignalValue(signal) {
  const subscribe = React.useCallback(
    (onChange) => v0_9.effect(() => {
      v0_9.getValue(signal);
      onChange();
    }),
    [signal]
  );
  const getSnapshot = React.useCallback(() => v0_9.peekValue(signal), [signal]);
  return React.useSyncExternalStore(subscribe, getSnapshot);
}
function registerChild(index, child) {
  index.set(child.id, child);
  return child.id;
}
function toViewValue(parent, value, index) {
  if (v0_9.isComponentNode(value)) {
    const token = registerChild(index, value);
    if (value.dataPath !== parent.dataPath) {
      return { id: token, basePath: value.dataPath };
    }
    return token;
  }
  if (value instanceof v0_9.ResolvedBinding) {
    return toViewValue(parent, value.value, index);
  }
  if (Array.isArray(value)) {
    return value.map((item) => toViewValue(parent, item, index));
  }
  if (isPlainObject(value)) {
    return toViewProps(parent, value, index);
  }
  return value;
}
function isPlainObject(value) {
  if (!value || typeof value !== "object") {
    return false;
  }
  const proto = Object.getPrototypeOf(value);
  return proto === Object.prototype || proto === null;
}
function toViewProps(parent, props, index) {
  const result = {};
  for (const [key, inner] of Object.entries(props)) {
    if (inner instanceof v0_9.ResolvedBinding) {
      result[key] = toViewValue(parent, inner.value, index);
      result[`set${key.charAt(0).toUpperCase()}${key.slice(1)}`] = v0_9.isWritable(inner) ? inner.set : () => {
      };
    } else {
      result[key] = toViewValue(parent, inner, index);
    }
  }
  return result;
}
function useNodeView(node, buildChild) {
  const context = contextOf(node);
  const surface = context.dataContext.surface;
  const resolved = useSignalValue(node.props);
  const { viewProps, childIndex } = React.useMemo(() => {
    const index = /* @__PURE__ */ new Map();
    return { viewProps: toViewProps(node, resolved, index), childIndex: index };
  }, [node, resolved]);
  const rawBuildChild = React.useCallback(
    (id, basePath) => {
      const requested = basePath ?? node.dataPath;
      const instances = [...childIndex.values()].filter((child) => child.componentId === id);
      const childNode = instances.find((child) => child.dataPath === requested);
      if (childNode) {
        return buildChild(childNode, basePath);
      }
      const elsewhere = [...new Set(instances.map((child) => child.dataPath))];
      if (elsewhere.length > 0) {
        return /* @__PURE__ */ jsxRuntime.jsx(
          UnresolvedChildReference,
          {
            surface,
            id,
            requestedPath: requested,
            detail: `instances exist at ${elsewhere.join(", ")}. Instances are created only at the data paths the payload implies; buildChild selects among them.`
          },
          JSON.stringify([id, requested])
        );
      }
      return buildChild(id, basePath);
    },
    [buildChild, node, surface, childIndex]
  );
  const viewBuildChild = React.useCallback(
    (id, basePath) => {
      const childNode = childIndex.get(id);
      return childNode ? buildChild(childNode, basePath) : rawBuildChild(id, basePath);
    },
    [buildChild, rawBuildChild, childIndex]
  );
  return { viewProps, context, viewBuildChild, rawBuildChild };
}
var RenderFallback = ({ node, impl, buildChild }) => {
  const { context, rawBuildChild } = useNodeView(node, buildChild);
  const Render = impl.render;
  return /* @__PURE__ */ jsxRuntime.jsx(Render, { context, buildChild: rawBuildChild });
};
var ChildElement = React.memo(({ node }) => {
  const owner = React.useMemo(() => new HostOwner(), []);
  if (node.state === "unknown-type") {
    return /* @__PURE__ */ jsxRuntime.jsxs("div", { style: { color: "red" }, children: [
      "Unknown component type: ",
      node.type
    ] });
  }
  if (node.isPlaceholder || !node.impl || !node.context) {
    return /* @__PURE__ */ jsxRuntime.jsx(LoadingPlaceholder, { componentId: node.componentId });
  }
  if ("render" in node.impl) {
    return /* @__PURE__ */ jsxRuntime.jsx(WebComponentNode, { node, owner, children: /* @__PURE__ */ jsxRuntime.jsx(NodeContent, { node }) });
  }
  const surface = node.context.dataContext.surface;
  return /* @__PURE__ */ jsxRuntime.jsxs(jsxRuntime.Fragment, { children: [
    /* @__PURE__ */ jsxRuntime.jsx(WebComponentNode, { node, owner }),
    /* @__PURE__ */ jsxRuntime.jsx(HostedChildren, { surface, owner })
  ] });
});
ChildElement.displayName = "ChildElement";
var HostedChildren = ({ surface, owner }) => {
  const registry = HostRegistry.forSurface(surface);
  const subscribe = React.useCallback(
    (listener) => registry.subscribe(owner, listener),
    [registry, owner]
  );
  const getSnapshot = React.useCallback(() => registry.getSnapshot(owner), [registry, owner]);
  const hosts = React.useSyncExternalStore(subscribe, getSnapshot);
  return /* @__PURE__ */ jsxRuntime.jsx(jsxRuntime.Fragment, { children: hosts.map(({ host, node }) => reactDom.createPortal(/* @__PURE__ */ jsxRuntime.jsx(NodeContent, { node }), host, node.id)) });
};
var NodeContent = React.memo(({ node }) => {
  const surface = contextOf(node).dataContext.surface;
  const buildChild = React.useCallback(
    (child, basePath) => {
      if (v0_9.isComponentNode(child)) {
        return /* @__PURE__ */ jsxRuntime.jsx(ChildElement, { node: child }, child.id);
      }
      const requested = basePath ?? node.dataPath;
      const detail = surface.componentsModel.get(child) ? "the component exists, but the catalog schema does not mark the referencing property as a component id. Use componentId() or childList() from @a2ui/web_core." : "no component with this id exists on the surface.";
      return /* @__PURE__ */ jsxRuntime.jsx(
        UnresolvedChildReference,
        {
          surface,
          id: child,
          requestedPath: requested,
          detail
        },
        JSON.stringify([child, requested])
      );
    },
    [surface, node]
  );
  const impl = node.impl;
  if (node.disposed || !impl || !("render" in impl)) {
    return null;
  }
  const View = impl.view;
  if (!View) {
    return /* @__PURE__ */ jsxRuntime.jsx(RenderFallback, { node, impl, buildChild });
  }
  return /* @__PURE__ */ jsxRuntime.jsx(View, { node, buildChild });
});
NodeContent.displayName = "NodeContent";
var MarkdownContext = React.createContext(void 0);
var useMarkdownRenderer = () => React.useContext(MarkdownContext);
var A2uiSurface = ({ surface }) => {
  const markdownRenderer = useMarkdownRenderer();
  React.useLayoutEffect(() => {
    if (markdownRenderer) {
      basic_catalog.setMarkdownRenderer(markdownRenderer);
    }
  }, [markdownRenderer]);
  const box = React.useMemo(
    () => ({ resolver: void 0 }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [surface]
  );
  const subscribe = React.useCallback(
    (onChange) => {
      prepareCatalogs(surface);
      const resolver = new v0_9.NodeResolver(surface, surface.defaultCatalog);
      box.resolver = resolver;
      const stopEffect = v0_9.effect(() => {
        v0_9.getValue(resolver.rootNode);
        onChange();
      });
      return () => {
        stopEffect();
        resolver.dispose();
        if (box.resolver === resolver) {
          box.resolver = void 0;
        }
      };
    },
    [surface, box]
  );
  const getSnapshot = React.useCallback(
    () => box.resolver ? v0_9.peekValue(box.resolver.rootNode) : void 0,
    [box]
  );
  const root = React.useSyncExternalStore(subscribe, getSnapshot);
  if (!root) {
    return /* @__PURE__ */ jsxRuntime.jsx(LoadingPlaceholder, { componentId: "root" });
  }
  return /* @__PURE__ */ jsxRuntime.jsxs(jsxRuntime.Fragment, { children: [
    /* @__PURE__ */ jsxRuntime.jsx(ChildElement, { node: root }),
    /* @__PURE__ */ jsxRuntime.jsx(HostedChildren, { surface, owner: null })
  ] });
};
function createComponentImplementation(api, RenderComponent) {
  const MemoizedRender = React.memo(RenderComponent, (prev, next) => {
    if (prev.props !== next.props) return false;
    if (prev.buildChild !== next.buildChild) return false;
    if (prev.context.componentModel.id !== next.context.componentModel.id) return false;
    if (prev.context.dataContext.path !== next.context.dataContext.path) return false;
    return true;
  });
  const ReactWrapper = ({ context, buildChild }) => {
    const bindingRef = React.useRef(null);
    if (!bindingRef.current) {
      bindingRef.current = new v0_9.GenericBinder(context, api.schema);
    } else if (bindingRef.current.context !== context) {
      bindingRef.current.dispose();
      bindingRef.current = new v0_9.GenericBinder(context, api.schema);
    }
    const binding = bindingRef.current;
    const subscribe = React.useCallback(
      (callback) => {
        const sub = binding.subscribe(callback);
        return () => sub.unsubscribe();
      },
      [binding]
    );
    const getSnapshot = React.useCallback(() => binding.snapshot, [binding]);
    const props = React.useSyncExternalStore(subscribe, getSnapshot);
    React.useEffect(() => {
      return () => binding.dispose();
    }, [binding]);
    return /* @__PURE__ */ jsxRuntime.jsx(MemoizedRender, { props: props || {}, buildChild, context });
  };
  const NodeView = ({ node, buildChild }) => {
    const { viewProps, context, viewBuildChild } = useNodeView(node, buildChild);
    return /* @__PURE__ */ jsxRuntime.jsx(MemoizedRender, { props: viewProps, buildChild: viewBuildChild, context });
  };
  NodeView.displayName = `${api.name}.view`;
  return {
    name: api.name,
    schema: api.schema,
    render: ReactWrapper,
    view: NodeView
  };
}
function createBinderlessComponentImplementation(api, RenderComponent) {
  const NodeView = ({ node, buildChild }) => {
    const { context, rawBuildChild } = useNodeView(node, buildChild);
    return /* @__PURE__ */ jsxRuntime.jsx(RenderComponent, { context, buildChild: rawBuildChild });
  };
  NodeView.displayName = `${api.name}.view`;
  return {
    name: api.name,
    schema: api.schema,
    render: RenderComponent,
    view: NodeView
  };
}

exports.A2uiSurface = A2uiSurface;
exports.MarkdownContext = MarkdownContext;
exports.createBinderlessComponentImplementation = createBinderlessComponentImplementation;
exports.createComponentImplementation = createComponentImplementation;
exports.useMarkdownRenderer = useMarkdownRenderer;
exports.useSignalValue = useSignalValue;
//# sourceMappingURL=chunk-ZKLSRTW6.cjs.map
//# sourceMappingURL=chunk-ZKLSRTW6.cjs.map