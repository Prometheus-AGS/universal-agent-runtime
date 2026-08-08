const isMarkdownEngineModule = (moduleId) => /[\\/]node_modules[\\/](?:mermaid|shiki|@shikijs[\\/]|@mermaid-js[\\/])/u.test(moduleId);
const packageRelativeModuleId = (moduleId) => {
    const normalized = moduleId.replaceAll("\\", "/");
    const packageRoot = normalized.lastIndexOf("/node_modules/");
    return packageRoot >= 0 ? normalized.slice(packageRoot + "/node_modules/".length) : normalized;
};
/** Emit auditable chunk-to-module metadata for the lazy Markdown engine gate. */
export const markdownEngineGraphPlugin = () => ({
    name: "uar-markdown-engine-graph",
    apply: "build",
    generateBundle(_options, bundle) {
        const chunks = Object.values(bundle)
            .filter((output) => output.type === "chunk")
            .sort((left, right) => left.fileName.localeCompare(right.fileName));
        const graph = Object.fromEntries(chunks.map((chunk) => [
            chunk.fileName,
            {
                imports: [...chunk.imports].sort(),
                dynamicImports: [...chunk.dynamicImports].sort(),
                engineModules: Object.keys(chunk.modules)
                    .filter(isMarkdownEngineModule)
                    .map(packageRelativeModuleId)
                    .sort(),
            },
        ]));
        this.emitFile({
            type: "asset",
            fileName: ".vite/markdown-engine-graph.json",
            source: JSON.stringify({ schemaVersion: 1, chunks: graph }, null, 2),
        });
    },
});
