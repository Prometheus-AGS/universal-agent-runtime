import React3, { createContext, memo, useCallback, useEffect, useState, useContext, useMemo, useRef, useSyncExternalStore } from 'react';
import { isComponentNode, Catalog, ComponentContext, GenericBinder, effect, getValue, peekValue, ResolvedBinding, isWritable, NodeResolver } from '@a2ui/web_core/v0_9';
import { jsx, jsxs, Fragment } from 'react/jsx-runtime';
import { BASIC_FUNCTIONS, BasicCatalogThemeSchema, TextApi, ImageApi, IconApi, VideoApi, AudioPlayerApi, RowApi, ColumnApi, ListApi, CardApi, TabsApi, DividerApi, ModalApi, ButtonApi, TextFieldApi, CheckBoxApi, ChoicePickerApi, SliderApi, DateTimeInputApi, injectBasicCatalogStyles } from '@a2ui/web_core/v0_9/basic_catalog';

// src/v0_9/A2uiSurface.tsx
var NodeSurfaceContext = createContext(
  null
);
var LoadingPlaceholder = ({ componentId }) => /* @__PURE__ */ jsxs("div", { style: { color: "gray", padding: "4px" }, children: [
  "[Loading ",
  componentId,
  "...]"
] });
var reportedUnresolved = /* @__PURE__ */ new WeakMap();
var UnresolvedChildReference = ({ surface, id, requestedPath, detail }) => {
  const message = `Unresolved child reference '${id}' at '${requestedPath}': ${detail}`;
  useEffect(() => {
    if (!surface) {
      return;
    }
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
  return /* @__PURE__ */ jsx("div", { style: { color: "red" }, children: message });
};
function useSignalValue(signal) {
  const subscribe = useCallback(
    (onChange) => effect(() => {
      getValue(signal);
      onChange();
    }),
    [signal]
  );
  const getSnapshot = useCallback(() => peekValue(signal), [signal]);
  return useSyncExternalStore(subscribe, getSnapshot);
}
function newChildIndex() {
  return { byToken: /* @__PURE__ */ new Map(), byId: /* @__PURE__ */ new Map() };
}
function setChild(map, id, child, firstWins) {
  let byPath = map.get(id);
  if (!byPath) {
    byPath = /* @__PURE__ */ new Map();
    map.set(id, byPath);
  }
  if (!firstWins || !byPath.has(child.dataPath)) {
    byPath.set(child.dataPath, child);
  }
}
function registerChild(index, child) {
  setChild(index.byToken, child.instanceId, child, false);
  setChild(index.byId, child.componentId, child, true);
  return child.instanceId;
}
function toViewValue(parent, value, index) {
  if (isComponentNode(value)) {
    const token = registerChild(index, value);
    if (value.dataPath !== parent.dataPath) {
      return { id: token, basePath: value.dataPath };
    }
    return token;
  }
  if (value instanceof ResolvedBinding) {
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
    if (inner instanceof ResolvedBinding) {
      result[key] = toViewValue(parent, inner.value, index);
      result[`set${key.charAt(0).toUpperCase()}${key.slice(1)}`] = isWritable(inner) ? inner.set : () => {
      };
    } else {
      result[key] = toViewValue(parent, inner, index);
    }
  }
  return result;
}
function useNodeView(node, buildChild) {
  const surface = useContext(NodeSurfaceContext);
  const resolved = useSignalValue(node.props);
  const { viewProps, childIndex } = useMemo(() => {
    const index = newChildIndex();
    return { viewProps: toViewProps(node, resolved, index), childIndex: index };
  }, [node, resolved]);
  const context = useMemo(
    () => surface && surface.componentsModel.get(node.componentId) ? new ComponentContext(surface, node.componentId, node.dataPath) : void 0,
    [surface, node]
  );
  const resolveThrough = useCallback(
    (map, id, basePath) => {
      const requested = basePath ?? node.dataPath;
      const byPath = map.get(id);
      const childNode = byPath?.get(requested);
      if (childNode) {
        return buildChild(childNode, basePath);
      }
      const elsewhere = byPath ? [...byPath.keys()] : [];
      if (elsewhere.length > 0) {
        return /* @__PURE__ */ jsx(
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
    [buildChild, node, surface]
  );
  const viewBuildChild = useCallback(
    (id, basePath) => resolveThrough(childIndex.byToken, id, basePath),
    [resolveThrough, childIndex]
  );
  const rawBuildChild = useCallback(
    (id, basePath) => resolveThrough(childIndex.byId, id, basePath),
    [resolveThrough, childIndex]
  );
  if (!surface) {
    throw new Error("A2UI component views render only inside A2uiSurface.");
  }
  return { viewProps, context, viewBuildChild, rawBuildChild };
}
var RenderFallback = ({ node, impl, buildChild }) => {
  const { context, rawBuildChild } = useNodeView(node, buildChild);
  const Render = impl.render;
  if (!context) {
    return /* @__PURE__ */ jsx(LoadingPlaceholder, { componentId: node.componentId });
  }
  return /* @__PURE__ */ jsx(Render, { context, buildChild: rawBuildChild });
};
var NodeView = memo(
  ({
    surface,
    node
  }) => {
    const buildChild = useCallback(
      (child, basePath) => {
        if (isComponentNode(child)) {
          return /* @__PURE__ */ jsx(NodeView, { surface, node: child }, child.instanceId);
        }
        const requested = basePath ?? node.dataPath;
        const detail = surface.componentsModel.get(child) ? "the component exists, but the catalog schema does not mark the referencing property as a component id. Use componentId() or childList() from @a2ui/web_core." : "no component with this id exists on the surface.";
        return /* @__PURE__ */ jsx(
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
    if (node.state === "unknown-type") {
      return /* @__PURE__ */ jsxs("div", { style: { color: "red" }, children: [
        "Unknown component type: ",
        node.type
      ] });
    }
    if (node.isPlaceholder) {
      return /* @__PURE__ */ jsx(LoadingPlaceholder, { componentId: node.componentId });
    }
    const impl = node.impl;
    if (!impl) {
      return null;
    }
    const View = impl.view;
    if (!View) {
      return /* @__PURE__ */ jsx(RenderFallback, { node, impl, buildChild });
    }
    return /* @__PURE__ */ jsx(View, { node, buildChild });
  }
);
NodeView.displayName = "NodeView";
var A2uiSurface = ({ surface }) => {
  const box = useMemo(
    () => ({ resolver: void 0 }),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [surface]
  );
  const subscribe = useCallback(
    (onChange) => {
      const resolver = new NodeResolver(surface, surface.defaultCatalog);
      box.resolver = resolver;
      const stopEffect = effect(() => {
        getValue(resolver.rootNode);
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
  const getSnapshot = useCallback(
    () => box.resolver ? peekValue(box.resolver.rootNode) : void 0,
    [box]
  );
  const root = useSyncExternalStore(subscribe, getSnapshot);
  if (!root) {
    return /* @__PURE__ */ jsx(LoadingPlaceholder, { componentId: "root" });
  }
  return /* @__PURE__ */ jsx(NodeSurfaceContext.Provider, { value: surface, children: /* @__PURE__ */ jsx(NodeView, { surface, node: root }) });
};
function createComponentImplementation(api, RenderComponent) {
  const MemoizedRender = memo(RenderComponent, (prev, next) => {
    if (prev.props !== next.props) return false;
    if (prev.buildChild !== next.buildChild) return false;
    if (prev.context.componentModel.id !== next.context.componentModel.id) return false;
    if (prev.context.dataContext.path !== next.context.dataContext.path) return false;
    return true;
  });
  const ReactWrapper = ({ context, buildChild }) => {
    const bindingRef = useRef(null);
    if (!bindingRef.current) {
      bindingRef.current = new GenericBinder(context, api.schema);
    } else if (bindingRef.current.context !== context) {
      bindingRef.current.dispose();
      bindingRef.current = new GenericBinder(context, api.schema);
    }
    const binding = bindingRef.current;
    const subscribe = useCallback(
      (callback) => {
        const sub = binding.subscribe(callback);
        return () => sub.unsubscribe();
      },
      [binding]
    );
    const getSnapshot = useCallback(() => binding.snapshot, [binding]);
    const props = useSyncExternalStore(subscribe, getSnapshot);
    useEffect(() => {
      return () => binding.dispose();
    }, [binding]);
    return /* @__PURE__ */ jsx(MemoizedRender, { props: props || {}, buildChild, context });
  };
  const NodeView2 = ({ node, buildChild }) => {
    const { viewProps, context, viewBuildChild } = useNodeView(node, buildChild);
    if (!context) {
      return /* @__PURE__ */ jsx(LoadingPlaceholder, { componentId: node.componentId });
    }
    return /* @__PURE__ */ jsx(MemoizedRender, { props: viewProps, buildChild: viewBuildChild, context });
  };
  NodeView2.displayName = `${api.name}.view`;
  return {
    name: api.name,
    schema: api.schema,
    render: ReactWrapper,
    view: NodeView2
  };
}
function createBinderlessComponentImplementation(api, RenderComponent) {
  const NodeView2 = ({ node, buildChild }) => {
    const { context, rawBuildChild } = useNodeView(node, buildChild);
    if (!context) {
      return /* @__PURE__ */ jsx(LoadingPlaceholder, { componentId: node.componentId });
    }
    return /* @__PURE__ */ jsx(RenderComponent, { context, buildChild: rawBuildChild });
  };
  NodeView2.displayName = `${api.name}.view`;
  return {
    name: api.name,
    schema: api.schema,
    render: RenderComponent,
    view: NodeView2
  };
}
var useBasicCatalogStyles = () => {
  useEffect(() => {
    if (typeof document !== "undefined" && document.adoptedStyleSheets) {
      injectBasicCatalogStyles();
    }
  }, []);
};
var mapJustify = (j) => {
  switch (j) {
    case "center":
      return "center";
    case "end":
      return "flex-end";
    case "spaceAround":
      return "space-around";
    case "spaceBetween":
      return "space-between";
    case "spaceEvenly":
      return "space-evenly";
    case "start":
      return "flex-start";
    case "stretch":
      return "stretch";
    default:
      return "flex-start";
  }
};
var mapAlign = (a) => {
  switch (a) {
    case "start":
      return "flex-start";
    case "center":
      return "center";
    case "end":
      return "flex-end";
    case "stretch":
      return "stretch";
    default:
      return "stretch";
  }
};
var getBaseLeafStyle = () => ({
  boxSizing: "border-box"
});
var getBaseContainerStyle = () => ({
  boxSizing: "border-box"
});
var getWeightStyle = (weight) => {
  if (typeof weight !== "number") return {};
  return { flex: `${weight}`, minWidth: 0, minHeight: 0 };
};
var MarkdownContext = createContext(void 0);
var useMarkdownRenderer = () => useContext(MarkdownContext);

// src/v0_9/catalog/basic/hooks/useMarkdown.ts
var warningLogged = false;
function useMarkdown(text, options) {
  const renderer = useMarkdownRenderer();
  const [html, setHtml] = useState(null);
  const optionsKey = JSON.stringify(options);
  useEffect(() => {
    if (!renderer) {
      if (!warningLogged) {
        console.warn(
          "[useMarkdown]",
          "can't render markdown because no markdown renderer is configured.\n",
          "Use `@a2ui/markdown-it`, or your own markdown renderer."
        );
        warningLogged = true;
      }
      setHtml(null);
      return;
    }
    let active = true;
    const parsedOptions = optionsKey ? JSON.parse(optionsKey) : void 0;
    renderer(text, parsedOptions).then((result) => {
      if (active) {
        setHtml(result);
      }
    }).catch((err) => {
      console.error("[useMarkdown] Render failed:", err);
    });
    return () => {
      active = false;
    };
  }, [text, renderer, optionsKey]);
  return html;
}

// src/v0_9/catalog/basic/components/Text.module.css
var Text_default = {};
var NON_MARKDOWN_VARIANTS = /* @__PURE__ */ new Set(["h1", "h2", "h3", "h4", "h5", "caption"]);
function MarkdownText({
  text,
  className,
  style
}) {
  const renderedHtml = useMarkdown(text);
  const classes = renderedHtml === null ? `${className} no-markdown-renderer` : className;
  const contentProps = renderedHtml !== null ? { dangerouslySetInnerHTML: { __html: renderedHtml } } : { children: text };
  return /* @__PURE__ */ jsx("div", { className: classes, style, ...contentProps });
}
function NonMarkdownText({
  text,
  variant,
  className,
  style
}) {
  const isCaption = variant === "caption";
  const HeadingTag = isCaption ? "em" : variant;
  if (isCaption) {
    return /* @__PURE__ */ jsx("span", { className, style, children: /* @__PURE__ */ jsx(HeadingTag, { children: text }) });
  }
  return /* @__PURE__ */ jsx("div", { className, style, children: /* @__PURE__ */ jsx(HeadingTag, { children: text }) });
}
var Text = createComponentImplementation(TextApi, ({ props }) => {
  useBasicCatalogStyles();
  const text = typeof props.text === "string" ? props.text : String(props.text ?? "");
  const variant = props.variant;
  const style = {
    ...getBaseLeafStyle(),
    ...getWeightStyle(props.weight)
  };
  if (variant && NON_MARKDOWN_VARIANTS.has(variant)) {
    const isCaption = variant === "caption";
    const className2 = [Text_default.a2uiText, isCaption ? Text_default.a2uiCaption : variant].join(" ");
    return /* @__PURE__ */ jsx(NonMarkdownText, { text, variant, className: className2, style });
  }
  const className = [Text_default.a2uiText, variant || "body"].join(" ");
  return /* @__PURE__ */ jsx(MarkdownText, { text, className, style });
});
var Image = createComponentImplementation(ImageApi, ({ props }) => {
  useBasicCatalogStyles();
  const mapFit = (fit) => {
    if (fit === "scaleDown") return "scale-down";
    return fit || "fill";
  };
  const style = {
    ...getBaseLeafStyle(),
    ...getWeightStyle(props.weight),
    objectFit: mapFit(props.fit),
    display: "block",
    borderRadius: "var(--a2ui-image-border-radius, 0)"
  };
  if (props.variant === "icon") {
    style.width = "var(--a2ui-image-icon-size, 24px)";
    style.height = "var(--a2ui-image-icon-size, 24px)";
  } else if (props.variant === "avatar") {
    style.width = "var(--a2ui-image-avatar-size, 40px)";
    style.height = "var(--a2ui-image-avatar-size, 40px)";
    style.borderRadius = "50%";
  } else if (props.variant === "smallFeature") {
    style.maxWidth = "var(--a2ui-image-small-feature-size, 100px)";
  } else if (props.variant === "largeFeature") {
    style.maxHeight = "var(--a2ui-image-large-feature-size, 400px)";
  } else if (props.variant === "header") {
    style.height = "var(--a2ui-image-header-size, 200px)";
    style.objectFit = "cover";
  }
  return /* @__PURE__ */ jsx("img", { src: props.url, alt: props.description || "", style });
});
var ICON_NAME_OVERRIDES = {
  play: "play_arrow",
  rewind: "fast_rewind",
  favoriteOff: "favorite_border",
  starOff: "star_border"
};
function toMaterialSymbol(str) {
  return ICON_NAME_OVERRIDES[str] ?? str.replace(/[A-Z]/g, (letter) => "_" + letter.toLowerCase());
}
var Icon = createComponentImplementation(IconApi, ({ props }) => {
  useBasicCatalogStyles();
  const isPath = typeof props.name === "object" && props.name !== null && "svgPath" in props.name;
  const baseStyle = {
    ...getBaseLeafStyle(),
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    fontSize: "var(--a2ui-icon-size, var(--a2ui-font-size-xl, 24px))",
    color: "var(--a2ui-icon-color, inherit)",
    lineHeight: 1
  };
  if (isPath) {
    const path = props.name.svgPath;
    return /* @__PURE__ */ jsx(
      "svg",
      {
        className: "a2ui-icon svg",
        viewBox: "0 0 24 24",
        style: {
          ...baseStyle,
          fill: "currentColor",
          width: "var(--a2ui-icon-size, 24px)",
          height: "var(--a2ui-icon-size, 24px)"
        },
        children: /* @__PURE__ */ jsx("path", { d: path })
      }
    );
  }
  const iconName = typeof props.name === "string" ? toMaterialSymbol(props.name) : "";
  const fontStyle = {
    ...baseStyle,
    fontFamily: 'var(--a2ui-icon-font-family, "Material Symbols Outlined", sans-serif)',
    fontVariationSettings: 'var(--a2ui-icon-font-variation-settings, "FILL" 1)',
    fontWeight: "normal",
    fontStyle: "normal",
    letterSpacing: "normal",
    textTransform: "none"
  };
  return /* @__PURE__ */ jsx("span", { className: "material-symbols-outlined", style: fontStyle, children: iconName });
});
var Video = createComponentImplementation(VideoApi, ({ props }) => {
  useBasicCatalogStyles();
  const style = {
    ...getBaseLeafStyle(),
    width: "100%",
    height: "auto",
    borderRadius: "var(--a2ui-video-border-radius, 0)"
  };
  return /* @__PURE__ */ jsx("video", { src: props.url, controls: true, style });
});
var AudioPlayer = createComponentImplementation(AudioPlayerApi, ({ props }) => {
  useBasicCatalogStyles();
  const containerStyle = {
    display: "flex",
    flexDirection: "column",
    gap: "var(--a2ui-spacing-xs, 0.25rem)",
    background: "var(--a2ui-audioplayer-background, transparent)",
    borderRadius: "var(--a2ui-audioplayer-border-radius, 0)",
    padding: "var(--a2ui-audioplayer-padding, 0)"
  };
  return /* @__PURE__ */ jsxs("div", { style: containerStyle, children: [
    props.description && /* @__PURE__ */ jsx(
      "span",
      {
        style: {
          fontSize: "var(--a2ui-font-size-xs, 0.75rem)",
          color: "var(--a2ui-text-caption-color, light-dark(#666, #aaa))"
        },
        children: props.description
      }
    ),
    /* @__PURE__ */ jsx("audio", { src: props.url, controls: true })
  ] });
});
var ChildList = ({ childList, buildChild }) => {
  if (!Array.isArray(childList)) return null;
  return /* @__PURE__ */ jsx(Fragment, { children: childList.map((childRef, index) => {
    if (typeof childRef === "string") {
      return /* @__PURE__ */ jsx(React3.Fragment, { children: buildChild(childRef) }, `${childRef}-${index}`);
    }
    return /* @__PURE__ */ jsx(React3.Fragment, { children: buildChild(childRef.id, childRef.basePath) }, `${childRef.id}-${childRef.basePath}`);
  }) });
};
var Row = createComponentImplementation(RowApi, ({ props, buildChild, context }) => {
  useBasicCatalogStyles();
  return /* @__PURE__ */ jsx(
    "div",
    {
      style: {
        ...getWeightStyle(props.weight),
        display: "flex",
        flexDirection: "row",
        justifyContent: mapJustify(props.justify),
        alignItems: mapAlign(props.align),
        gap: "var(--a2ui-row-gap, var(--a2ui-spacing-m))"
      },
      children: /* @__PURE__ */ jsx(ChildList, { childList: props.children, buildChild, context })
    }
  );
});
var Column = createComponentImplementation(ColumnApi, ({ props, buildChild, context }) => {
  useBasicCatalogStyles();
  return /* @__PURE__ */ jsx(
    "div",
    {
      style: {
        ...getWeightStyle(props.weight),
        display: "flex",
        flexDirection: "column",
        justifyContent: mapJustify(props.justify),
        alignItems: mapAlign(props.align),
        gap: "var(--a2ui-column-gap, var(--a2ui-spacing-m))"
      },
      children: /* @__PURE__ */ jsx(ChildList, { childList: props.children, buildChild, context })
    }
  );
});
var List = createComponentImplementation(ListApi, ({ props, buildChild, context }) => {
  useBasicCatalogStyles();
  const isHorizontal = props.direction === "horizontal";
  const style = {
    display: "flex",
    flexDirection: isHorizontal ? "row" : "column",
    alignItems: mapAlign(props.align),
    overflowX: isHorizontal ? "auto" : "hidden",
    overflowY: isHorizontal ? "hidden" : "auto",
    gap: "var(--a2ui-list-gap, var(--a2ui-spacing-s))",
    padding: "var(--a2ui-list-padding, 0)"
  };
  return /* @__PURE__ */ jsx("div", { style, children: /* @__PURE__ */ jsx(ChildList, { childList: props.children, buildChild, context }) });
});
var Card = createComponentImplementation(CardApi, ({ props, buildChild }) => {
  useBasicCatalogStyles();
  const style = {
    ...getBaseContainerStyle(),
    ...getWeightStyle(props.weight),
    display: "block",
    border: "var(--a2ui-card-border, var(--a2ui-border))",
    borderRadius: "var(--a2ui-card-border-radius, var(--a2ui-border-radius, 8px))",
    padding: "var(--a2ui-card-padding, var(--a2ui-spacing-m, 16px))",
    background: "var(--a2ui-card-background, var(--a2ui-color-surface, #fff))",
    color: "var(--a2ui-color-on-surface, #333)",
    boxShadow: "var(--a2ui-card-box-shadow, 0 2px 4px rgba(0,0,0,0.1))",
    margin: "var(--a2ui-card-margin, var(--a2ui-spacing-m))"
  };
  return /* @__PURE__ */ jsx("div", { className: "a2ui-card", style, children: props.child ? buildChild(props.child) : null });
});
var Tabs = createComponentImplementation(TabsApi, ({ props, buildChild }) => {
  useBasicCatalogStyles();
  const [selectedIndex, setSelectedIndex] = useState(0);
  const tabs = props.tabs || [];
  const activeTab = tabs[selectedIndex];
  const tabsContainer = {
    display: "block"
  };
  const tabsHeaders = {
    display: "flex",
    gap: "var(--a2ui-spacing-xs, 0.25rem)",
    borderBottom: "var(--a2ui-tabs-border, var(--a2ui-border-width, 1px) solid var(--a2ui-color-border, #ccc))",
    marginBottom: "var(--a2ui-spacing-m, 0.5rem)"
  };
  const tabsHeaderBase = {
    padding: "var(--a2ui-spacing-m, 0.5rem) var(--a2ui-spacing-l, 1rem)",
    background: "var(--a2ui-tabs-header-background, transparent)",
    color: "var(--a2ui-tabs-header-color, var(--a2ui-color-on-surface))",
    border: "none",
    borderRadius: "var(--a2ui-border-radius, 0.25rem) var(--a2ui-border-radius, 0.25rem) 0 0",
    cursor: "pointer",
    fontFamily: "inherit"
  };
  const tabsHeaderActive = {
    background: "var(--a2ui-tabs-header-background-active, var(--a2ui-color-secondary, #eee))",
    color: "var(--a2ui-tabs-header-color-active, var(--a2ui-color-on-secondary, #333))"
  };
  const content = {
    padding: "var(--a2ui-tabs-content-padding, 0 var(--a2ui-spacing-m, 0.5rem))"
  };
  return /* @__PURE__ */ jsxs("div", { style: tabsContainer, children: [
    /* @__PURE__ */ jsx("div", { style: tabsHeaders, children: tabs.map((tab, i) => /* @__PURE__ */ jsx(
      "button",
      {
        className: `a2ui-tabs-header a2ui-tab-button ${selectedIndex === i ? "active" : ""}`,
        onClick: () => setSelectedIndex(i),
        style: {
          ...tabsHeaderBase,
          ...selectedIndex === i ? tabsHeaderActive : {}
        },
        children: tab.title
      },
      i
    )) }),
    /* @__PURE__ */ jsx("div", { style: content, children: activeTab ? buildChild(activeTab.child) : null })
  ] });
});
var Divider = createComponentImplementation(DividerApi, ({ props }) => {
  useBasicCatalogStyles();
  const isVertical = props.axis === "vertical";
  const style = {
    border: "none",
    backgroundColor: "var(--a2ui-color-border, #ccc)"
  };
  if (isVertical) {
    style.width = "var(--a2ui-border-width, 1px)";
    style.height = "100%";
    style.margin = "0 var(--a2ui-divider-spacing, var(--a2ui-spacing-m, 0.5rem))";
  } else {
    style.width = "100%";
    style.height = "var(--a2ui-border-width, 1px)";
    style.margin = "var(--a2ui-divider-spacing, var(--a2ui-spacing-m, 0.5rem)) 0";
  }
  return /* @__PURE__ */ jsx("div", { style });
});
var Modal = createComponentImplementation(ModalApi, ({ props, buildChild }) => {
  useBasicCatalogStyles();
  const [isOpen, setIsOpen] = useState(false);
  return /* @__PURE__ */ jsxs(Fragment, { children: [
    /* @__PURE__ */ jsx(
      "div",
      {
        className: "a2ui-modal-trigger",
        onClick: () => setIsOpen(true),
        style: { display: "inline-block" },
        children: props.trigger ? buildChild(props.trigger) : null
      }
    ),
    isOpen && /* @__PURE__ */ jsx(
      "div",
      {
        className: "a2ui-modal-overlay",
        style: {
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: "var(--a2ui-modal-overlay-color, rgba(0, 0, 0, 0.5))",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          zIndex: 1e3
        },
        onClick: () => setIsOpen(false),
        children: /* @__PURE__ */ jsxs(
          "div",
          {
            style: {
              backgroundColor: "var(--a2ui-color-surface, #fff)",
              padding: "var(--a2ui-modal-padding, var(--a2ui-spacing-l, 24px))",
              borderRadius: "var(--a2ui-modal-border-radius, var(--a2ui-border-radius, 8px))",
              maxWidth: "90%",
              maxHeight: "90%",
              overflow: "auto",
              display: "flex",
              flexDirection: "column",
              color: "var(--a2ui-color-on-surface, inherit)"
            },
            onClick: (e) => e.stopPropagation(),
            children: [
              /* @__PURE__ */ jsx("div", { style: { display: "flex", justifyContent: "flex-end" }, children: /* @__PURE__ */ jsx(
                "button",
                {
                  className: "a2ui-modal-close",
                  onClick: () => setIsOpen(false),
                  style: {
                    border: "none",
                    background: "none",
                    fontSize: "var(--a2ui-font-size-xl, 1.5rem)",
                    cursor: "pointer",
                    padding: "var(--a2ui-spacing-xs, 4px)",
                    color: "var(--a2ui-color-on-surface, inherit)"
                  },
                  children: "\xD7"
                }
              ) }),
              /* @__PURE__ */ jsx("div", { style: { flex: 1 }, children: props.content ? buildChild(props.content) : null })
            ]
          }
        )
      }
    )
  ] });
});

// src/v0_9/catalog/basic/components/Button.module.css
var Button_default = {};
var Button = createComponentImplementation(ButtonApi, ({ props, buildChild }) => {
  useBasicCatalogStyles();
  const classes = [Button_default.button];
  if (props.variant === "primary") {
    classes.push(Button_default.primary);
  } else if (props.variant === "borderless") {
    classes.push(Button_default.borderless);
  }
  return /* @__PURE__ */ jsx("button", { className: classes.join(" "), onClick: props.action, disabled: props.isValid === false, children: props.child ? buildChild(props.child) : null });
});

// src/v0_9/catalog/basic/components/TextField.module.css
var TextField_default = {};
var TextField = createComponentImplementation(TextFieldApi, ({ props }) => {
  useBasicCatalogStyles();
  const onChange = (e) => {
    props.setValue(e.target.value);
  };
  const isLong = props.variant === "longText";
  const type = props.variant === "number" ? "number" : props.variant === "obscured" ? "password" : "text";
  const uniqueId = React3.useId();
  const hasError = props.validationErrors && props.validationErrors.length > 0;
  const inputClasses = `${TextField_default.input} ${hasError ? TextField_default.invalid : ""}`;
  return /* @__PURE__ */ jsxs("div", { className: TextField_default.host, children: [
    props.label && /* @__PURE__ */ jsx("label", { htmlFor: uniqueId, className: TextField_default.label, children: props.label }),
    isLong ? /* @__PURE__ */ jsx(
      "textarea",
      {
        id: uniqueId,
        className: inputClasses,
        value: props.value || "",
        onChange
      }
    ) : /* @__PURE__ */ jsx(
      "input",
      {
        id: uniqueId,
        type,
        className: inputClasses,
        value: props.value || "",
        onChange
      }
    ),
    hasError && /* @__PURE__ */ jsx("span", { className: TextField_default.error, children: props.validationErrors[0] })
  ] });
});
var CheckBox = createComponentImplementation(CheckBoxApi, ({ props }) => {
  useBasicCatalogStyles();
  const onChange = (e) => {
    props.setValue(e.target.checked);
  };
  const uniqueId = React3.useId();
  const hasError = props.validationErrors && props.validationErrors.length > 0;
  const containerStyle = {
    display: "flex",
    flexDirection: "column",
    margin: "var(--a2ui-checkbox-margin, var(--a2ui-spacing-m))"
  };
  const rowStyle = {
    display: "flex",
    alignItems: "center",
    gap: "var(--a2ui-checkbox-gap, var(--a2ui-spacing-s, 0.5rem))"
  };
  const inputBaseStyle = {
    cursor: "pointer",
    width: "var(--a2ui-checkbox-size, 1rem)",
    height: "var(--a2ui-checkbox-size, 1rem)",
    background: "var(--a2ui-checkbox-background, inherit)",
    border: "var(--a2ui-checkbox-border, var(--a2ui-border))",
    borderRadius: "var(--a2ui-checkbox-border-radius, 4px)",
    outline: "none"
  };
  const inputErrorStyle = {
    outline: "1px solid var(--a2ui-checkbox-color-error, red)"
  };
  const labelBaseStyle = {
    cursor: "pointer",
    color: "var(--a2ui-color-on-surface, inherit)",
    fontSize: "var(--a2ui-checkbox-label-font-size, var(--a2ui-label-font-size, var(--a2ui-font-size-s)))",
    fontWeight: "var(--a2ui-checkbox-label-font-weight, var(--a2ui-label-font-weight, bold))"
  };
  const labelErrorStyle = {
    color: "var(--a2ui-checkbox-color-error, red)"
  };
  const errorStyle = {
    fontSize: "var(--a2ui-font-size-xs, 0.75rem)",
    color: "var(--a2ui-checkbox-color-error, red)",
    marginTop: "4px"
  };
  return /* @__PURE__ */ jsxs("div", { style: containerStyle, children: [
    /* @__PURE__ */ jsxs("div", { style: rowStyle, children: [
      /* @__PURE__ */ jsx(
        "input",
        {
          id: uniqueId,
          type: "checkbox",
          checked: !!props.value,
          onChange,
          style: {
            ...inputBaseStyle,
            ...hasError ? inputErrorStyle : {}
          }
        }
      ),
      props.label && /* @__PURE__ */ jsx(
        "label",
        {
          htmlFor: uniqueId,
          style: {
            ...labelBaseStyle,
            ...hasError ? labelErrorStyle : {}
          },
          children: props.label
        }
      )
    ] }),
    hasError && /* @__PURE__ */ jsx("span", { style: errorStyle, children: props.validationErrors?.[0] })
  ] });
});

// src/v0_9/catalog/basic/components/ChoicePicker.module.css
var ChoicePicker_default = {};
var ChoicePicker = createComponentImplementation(ChoicePickerApi, ({ props }) => {
  useBasicCatalogStyles();
  const [filter, setFilter] = useState("");
  const groupName = React3.useId();
  const values = Array.isArray(props.value) ? props.value : [];
  const isMutuallyExclusive = props.variant === "mutuallyExclusive";
  const onToggle = (val) => {
    if (isMutuallyExclusive) {
      props.setValue([val]);
    } else {
      const newValues = values.includes(val) ? values.filter((v) => v !== val) : [...values, val];
      props.setValue(newValues);
    }
  };
  const options = (props.options || []).filter(
    (opt) => !props.filterable || filter === "" || String(opt.label).toLowerCase().includes(filter.toLowerCase())
  );
  const listClasses = `${ChoicePicker_default.options} ${props.displayStyle === "chips" ? ChoicePicker_default.chips : ""}`;
  return /* @__PURE__ */ jsxs("div", { className: ChoicePicker_default.host, children: [
    props.label && /* @__PURE__ */ jsx("strong", { className: ChoicePicker_default.label, children: props.label }),
    props.filterable && /* @__PURE__ */ jsx(
      "input",
      {
        type: "text",
        placeholder: "Filter options...",
        value: filter,
        onChange: (e) => setFilter(e.target.value),
        className: ChoicePicker_default.filterInput
      }
    ),
    /* @__PURE__ */ jsx("div", { className: listClasses, children: options.map((opt, i) => {
      const isSelected = values.includes(opt.value);
      if (props.displayStyle === "chips") {
        return /* @__PURE__ */ jsx(
          "button",
          {
            onClick: () => onToggle(opt.value),
            className: `${ChoicePicker_default.chip} chip ${isSelected ? `${ChoicePicker_default.selected} selected` : ""}`,
            "aria-pressed": isSelected,
            children: opt.label
          },
          i
        );
      }
      return /* @__PURE__ */ jsxs("label", { className: ChoicePicker_default.optionLabel, children: [
        /* @__PURE__ */ jsx(
          "input",
          {
            type: isMutuallyExclusive ? "radio" : "checkbox",
            checked: isSelected,
            onChange: () => onToggle(opt.value),
            name: isMutuallyExclusive ? groupName : void 0
          }
        ),
        /* @__PURE__ */ jsx("span", { className: ChoicePicker_default.optionText, children: opt.label })
      ] }, i);
    }) })
  ] });
});
var Slider = createComponentImplementation(SliderApi, ({ props }) => {
  useBasicCatalogStyles();
  const onChange = (e) => {
    props.setValue(Number(e.target.value));
  };
  const uniqueId = React3.useId();
  const containerStyle = {
    display: "flex",
    flexDirection: "column",
    gap: "var(--a2ui-spacing-xs, 0.25rem)",
    margin: "var(--a2ui-slider-margin, var(--a2ui-spacing-m))"
  };
  const headerStyle = {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center"
  };
  const labelStyle = {
    fontSize: "var(--a2ui-slider-label-font-size, var(--a2ui-label-font-size, var(--a2ui-font-size-s)))",
    fontWeight: "var(--a2ui-slider-label-font-weight, var(--a2ui-label-font-weight, bold))"
  };
  const valueStyle = {
    fontSize: "var(--a2ui-font-size-xs, 0.75rem)",
    color: "var(--a2ui-text-caption-color, light-dark(#666, #aaa))"
  };
  const inputStyle = {
    width: "100%",
    cursor: "pointer",
    accentColor: "var(--a2ui-slider-thumb-color, var(--a2ui-color-primary, #007bff))",
    background: "var(--a2ui-slider-track-color, var(--a2ui-color-secondary, #e9ecef))"
  };
  return /* @__PURE__ */ jsxs("div", { style: containerStyle, children: [
    /* @__PURE__ */ jsxs("div", { style: headerStyle, children: [
      props.label && /* @__PURE__ */ jsx("label", { htmlFor: uniqueId, style: labelStyle, children: props.label }),
      /* @__PURE__ */ jsx("span", { style: valueStyle, children: props.value })
    ] }),
    /* @__PURE__ */ jsx(
      "input",
      {
        id: uniqueId,
        type: "range",
        min: props.min ?? 0,
        max: props.max,
        value: props.value ?? 0,
        onChange,
        style: inputStyle
      }
    )
  ] });
});
if (typeof document !== "undefined") {
  const styleId = "a2ui-date-time-input-webkit-styles";
  if (!document.getElementById(styleId)) {
    const style = document.createElement("style");
    style.id = styleId;
    style.textContent = `
      .a2ui-date-time-input::-webkit-datetime-edit,
      .a2ui-date-time-input::-webkit-datetime-edit-fields-wrapper {
        color: var(--a2ui-datetimeinput-color, var(--a2ui-color-on-input, #333));
      }
    `;
    document.head.appendChild(style);
  }
}
function normalizeDateTimeValue(value, type) {
  if (!value) return "";
  const hasT = value.includes("T");
  const split = value.split("T");
  const datePart = (hasT ? split[0] : value)?.substring(0, 10) ?? "";
  const timePart = (hasT ? split[1] : value)?.substring(0, 5) ?? "";
  switch (type) {
    case "date":
      return datePart;
    case "time":
      return timePart;
    case "datetime-local":
      return `${datePart}T${timePart}`;
  }
  return "";
}
var DateTimeInput = createComponentImplementation(DateTimeInputApi, ({ props }) => {
  useBasicCatalogStyles();
  const onChange = (e) => {
    props.setValue(e.target.value);
  };
  const uniqueId = React3.useId();
  if (!(props.enableDate || props.enableTime)) return null;
  let type = "datetime-local";
  if (props.enableDate && !props.enableTime) type = "date";
  if (!props.enableDate && props.enableTime) type = "time";
  const normalizedValue = normalizeDateTimeValue(props.value, type);
  const style = {
    backgroundColor: "var(--a2ui-datetimeinput-background, var(--a2ui-color-input, #fff))",
    color: "var(--a2ui-datetimeinput-color, var(--a2ui-color-on-input, #333))",
    border: "var(--a2ui-datetimeinput-border, var(--a2ui-border))",
    borderRadius: "var(--a2ui-datetimeinput-border-radius, var(--a2ui-border-radius))",
    padding: "var(--a2ui-datetimeinput-padding, var(--a2ui-spacing-s))",
    boxSizing: "border-box"
  };
  return /* @__PURE__ */ jsxs(
    "div",
    {
      style: {
        display: "flex",
        flexDirection: "column",
        gap: "var(--a2ui-spacing-xs, 0.25rem)"
      },
      children: [
        props.label && /* @__PURE__ */ jsx(
          "label",
          {
            htmlFor: uniqueId,
            style: {
              fontSize: "var(--a2ui-datetimeinput-label-font-size, var(--a2ui-label-font-size, var(--a2ui-font-size-s)))",
              fontWeight: "var(--a2ui-datetimeinput-label-font-weight, var(--a2ui-label-font-weight, bold))"
            },
            children: props.label
          }
        ),
        /* @__PURE__ */ jsx(
          "input",
          {
            id: uniqueId,
            className: "a2ui-date-time-input",
            type,
            style,
            value: normalizedValue,
            onChange,
            min: typeof props.min === "string" ? props.min : void 0,
            max: typeof props.max === "string" ? props.max : void 0
          }
        )
      ]
    }
  );
});

// src/v0_9/catalog/basic/index.ts
var basicComponents = [
  Text,
  Image,
  Icon,
  Video,
  AudioPlayer,
  Row,
  Column,
  List,
  Card,
  Tabs,
  Divider,
  Modal,
  Button,
  TextField,
  CheckBox,
  ChoicePicker,
  Slider,
  DateTimeInput
];
var basicCatalog = new Catalog(
  "https://a2ui.org/specification/v0_9/catalogs/basic/catalog.json",
  "0.9",
  basicComponents,
  BASIC_FUNCTIONS,
  BasicCatalogThemeSchema
);

export { A2uiSurface, AudioPlayer, Button, Card, CheckBox, ChoicePicker, Column, DateTimeInput, Divider, Icon, Image, List, MarkdownContext, Modal, Row, Slider, Tabs, Text, TextField, Video, basicCatalog, createBinderlessComponentImplementation, createComponentImplementation, useMarkdownRenderer, useSignalValue };
//# sourceMappingURL=index.js.map
//# sourceMappingURL=index.js.map