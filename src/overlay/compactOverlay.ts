export type CompactMonitor = {
  scaleFactor: number;
  position: { x: number; y: number };
  size: { width: number; height: number };
  workArea?: {
    position: { x: number; y: number };
    size: { width: number; height: number };
  };
};

export type PhysicalBounds = { x: number; y: number; width: number; height: number };

/** Place the small meeting marker at the center of the usable top edge of its monitor. */
export function compactOverlayBounds(
  monitor: CompactMonitor,
  logicalWidth = 148,
  logicalHeight = 88,
  topInset = 10,
): PhysicalBounds {
  const scale = Number.isFinite(monitor.scaleFactor) && monitor.scaleFactor > 0 ? monitor.scaleFactor : 1;
  const area = monitor.workArea ?? { position: monitor.position, size: monitor.size };
  const width = Math.round(logicalWidth * scale);
  const height = Math.round(logicalHeight * scale);
  return {
    x: Math.round(area.position.x + (area.size.width - width) / 2),
    y: Math.round(area.position.y + topInset * scale),
    width,
    height,
  };
}
