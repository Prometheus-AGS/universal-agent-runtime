import {
  useState as useLocalState,
  type Dispatch,
  type SetStateAction,
} from "react";

export function RenderBodySetter() {
  const [
    open,
    setOpen,
  ]: [boolean, Dispatch<SetStateAction<boolean>>] = useLocalState(false);
  if (!open) {
    setOpen(true);
  }
  return <span>{String(open)}</span>;
}
