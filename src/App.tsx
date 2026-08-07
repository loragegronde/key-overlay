import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { EditorApp } from "./apps/EditorApp";
import { OverlayApp } from "./apps/OverlayApp";

/**
 * One Vite build serves both windows. The label in tauri.conf.json decides
 * which shell mounts: the decorated editor, or the transparent HUD.
 */
export default function App() {
  const label = getCurrentWebviewWindow().label;
  return label === "editor" ? <EditorApp /> : <OverlayApp />;
}
