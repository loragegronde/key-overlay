import { useOverlayStore } from "../store/useOverlayStore";

/**
 * Isolated so that the KPS counter ticking does not re-render the whole
 * toolbar several times a second.
 */
export function KpsMeter({ className }: { className?: string }) {
  const kps = useOverlayStore((s) => s.kps);
  return <div className={className}>{kps} KPS</div>;
}
