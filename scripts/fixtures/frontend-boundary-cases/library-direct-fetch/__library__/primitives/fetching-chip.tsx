export async function FetchingChip() {
  const response = await fetch("/api/uar/providers");
  return <span>{response.status}</span>;
}
