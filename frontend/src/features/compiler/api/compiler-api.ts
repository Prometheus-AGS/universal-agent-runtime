export async function fetchCompilerSessions<T>(): Promise<T> {
  const res = await fetch("/api/compiler/sessions");
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json() as Promise<T>;
}

export async function createCompilerSession<T>(): Promise<T> {
  const res = await fetch("/api/compiler/sessions", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({}),
  });
  if (!res.ok) throw new Error(`${res.status}`);
  return res.json() as Promise<T>;
}
