import { getCurrentWindow } from "@tauri-apps/api/window";

export function WindowButtons() {
  return <button type="button" onClick={() => void getCurrentWindow().close()}>Close</button>;
}
