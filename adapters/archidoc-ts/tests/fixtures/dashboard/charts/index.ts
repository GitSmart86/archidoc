/**
 * @c4 component
 *
 * Real-time chart rendering with WebGL acceleration.
 *
 * @c4 uses engine "Chart data" "WebSocket"
 * @c4 uses theme_service "Theme configuration" "REST"
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `core.ts` | Facade | Chart API entry point | stable |
 * | `renderer.ts` | Strategy | Pluggable rendering backends | active |
 * | `theme.ts` | -- | Color and style definitions | stable |
 */
export { ChartCore } from './core';
export { Renderer } from './renderer';
export { Theme } from './theme';
