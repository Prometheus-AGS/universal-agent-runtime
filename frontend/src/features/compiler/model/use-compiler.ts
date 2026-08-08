import { useCompilerStore } from "./compiler-store";

export function useCompiler() {
  const loading = useCompilerStore((state) => state.loading);
  const creating = useCompilerStore((state) => state.creating);
  const error = useCompilerStore((state) => state.error);
  const load = useCompilerStore((state) => state.load);
  const createSession = useCompilerStore((state) => state.createSession);
  return { loading, creating, error, load, createSession };
}
