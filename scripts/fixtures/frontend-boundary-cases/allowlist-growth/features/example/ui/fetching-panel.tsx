export async function FetchingPanel() {
  const response = await fetch("/api/uar/providers");
  return <span>{response.status}</span>;
}
