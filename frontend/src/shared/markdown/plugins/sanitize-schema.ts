import { defaultSchema, type Options } from "rehype-sanitize";

const LIMITED_HTML_TAGS = [
  "figure",
  "figcaption",
  "mark",
  "video",
] as const;

const LIMITED_SVG_TAGS = [
  "svg",
  "path",
  "g",
  "defs",
  "marker",
  "polygon",
  "polyline",
  "circle",
  "ellipse",
  "rect",
  "line",
  "text",
  "tspan",
  "use",
  "symbol",
  "clipPath",
  "linearGradient",
  "stop",
] as const;

const appendUnique = <T,>(base: readonly T[] | null | undefined, additions: readonly T[]): T[] =>
  Array.from(new Set([...(base ?? []), ...additions]));

/**
 * Build the allowlist for untrusted markdown-derived HTML.
 *
 * The schema intentionally excludes `style`, event-handler attributes, and
 * executable/embed elements. KaTeX output is created only after this boundary.
 */
export const createMarkdownSanitizeSchema = (): Options => ({
  ...defaultSchema,
  tagNames: appendUnique(defaultSchema.tagNames, [
    ...LIMITED_HTML_TAGS,
    ...LIMITED_SVG_TAGS,
  ]),
  attributes: {
    ...defaultSchema.attributes,
    a: appendUnique(defaultSchema.attributes?.a, ["target", "rel"]),
    code: [
      ...(defaultSchema.attributes?.code ?? []),
      ["className", /^language-./, "math-inline", "math-display"],
    ],
    img: appendUnique(defaultSchema.attributes?.img, [
      "alt",
      "title",
      "width",
      "height",
      "loading",
    ]),
    video: [
      "src",
      "poster",
      "controls",
      "muted",
      "playsInline",
      "width",
      "height",
    ],
    source: appendUnique(defaultSchema.attributes?.source, ["src", "type"]),
    svg: [
      "viewBox",
      "xmlns",
      "width",
      "height",
      "fill",
      "stroke",
      "preserveAspectRatio",
      "role",
      "ariaLabel",
    ],
    path: ["d", "fill", "stroke", "strokeWidth", "transform"],
    g: ["fill", "stroke", "strokeWidth", "transform"],
    marker: ["id", "markerHeight", "markerUnits", "markerWidth", "orient", "refX", "refY", "viewBox"],
    polygon: ["fill", "points", "stroke", "strokeWidth", "transform"],
    polyline: ["fill", "points", "stroke", "strokeWidth", "transform"],
    circle: ["cx", "cy", "fill", "r", "stroke", "strokeWidth", "transform"],
    ellipse: ["cx", "cy", "fill", "rx", "ry", "stroke", "strokeWidth", "transform"],
    rect: ["fill", "height", "rx", "ry", "stroke", "strokeWidth", "transform", "width", "x", "y"],
    line: ["stroke", "strokeWidth", "transform", "x1", "x2", "y1", "y2"],
    text: ["fill", "textAnchor", "transform", "x", "y"],
    tspan: ["dx", "dy", "fill", "textAnchor", "x", "y"],
    use: ["href", "x", "y"],
    symbol: ["id", "preserveAspectRatio", "viewBox"],
    clipPath: ["id", "transform"],
    linearGradient: ["id", "gradientTransform", "x1", "x2", "y1", "y2"],
    stop: ["offset", "stopColor", "stopOpacity"],
  },
  protocols: {
    ...defaultSchema.protocols,
    href: ["http", "https", "mailto"],
    src: ["http", "https", "blob"],
  },
});

/** Immutable-by-convention schema shared by the configured rehype chain. */
export const markdownSanitizeSchema = createMarkdownSanitizeSchema();
