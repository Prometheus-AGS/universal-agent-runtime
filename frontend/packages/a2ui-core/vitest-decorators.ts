import ts from "typescript";

export function compileA2uiTestDecorators() {
  return {
    name: "test-compiled-a2ui-decorators",
    enforce: "pre" as const,
    transform(code: string, id: string) {
      const file = id.split("?", 1)[0];
      if (!file.includes("/vendor/a2ui-web-core/src/") || !file.endsWith(".ts") || !/^\s*@\w+\(/m.test(code)) {
        return;
      }
      // Vitest runs this source in Node, which cannot parse standard decorators.
      const compiled = ts.transpileModule(code, {
        fileName: file,
        compilerOptions: {
          target: ts.ScriptTarget.ES2022,
          module: ts.ModuleKind.ESNext,
          sourceMap: true,
        },
      });
      return { code: compiled.outputText, map: compiled.sourceMapText ?? null };
    },
  };
}
