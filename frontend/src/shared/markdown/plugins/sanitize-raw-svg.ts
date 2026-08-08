import createDOMPurify, { type WindowLike } from "dompurify";

/** Sanitize standalone SVG artifacts that do not pass through the markdown AST. */
export const sanitizeRawSvg = (source: string): string => {
  if (!createDOMPurify.isSupported && typeof window === "undefined") {
    return "";
  }

  const purifier = createDOMPurify.isSupported
    ? createDOMPurify
    : createDOMPurify(window as unknown as WindowLike);

  return purifier.sanitize(source, {
    USE_PROFILES: {
      svg: true,
      svgFilters: true,
    },
  });
};
