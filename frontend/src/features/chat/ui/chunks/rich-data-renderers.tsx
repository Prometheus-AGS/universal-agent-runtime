import { useAssistantDataUI, type DataMessagePartProps } from "@assistant-ui/react";
import type { FC } from "react";
import { CHUNK_BUBBLE_VISIBLE, type Chunk, type ChunkKind } from "@/features/chat/model/chunk";
import { ChunkRenderer } from "./chunk-renderer";

const ChunkDataPart: FC<DataMessagePartProps<Chunk>> = ({ data }) => <ChunkRenderer chunk={data} />;

const VISIBLE_CHUNK_DATA_PART_NAMES = (Object.entries(CHUNK_BUBBLE_VISIBLE) as Array<[ChunkKind, boolean]>)
  .filter(([, visible]) => visible)
  .map(([kind]) => kind);

function ChunkDataRegistration({ name }: { name: ChunkKind }) {
  useAssistantDataUI({ name, render: ChunkDataPart });
  return null;
}

/** Registers the complete visible chunk catalog with Assistant UI. */
export function RichDataRenderers() {
  return <>{VISIBLE_CHUNK_DATA_PART_NAMES.map((name) => <ChunkDataRegistration key={name} name={name} />)}</>;
}
