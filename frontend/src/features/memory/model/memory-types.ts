export interface MemoryItem {
  id: string;
  content: string;
  categories: string[];
  scope: string;
  memory_type: string;
  user_id?: string;
  agent_id?: string;
  session_id?: string;
  importance: number;
  created_at: string;
}

export interface MemoryStats {
  total: number;
  by_scope: Record<string, number>;
}

export interface MemoryListQuery {
  userId: string;
  agentId: string;
  searchQ: string;
  searchMode: boolean;
}
