/**
 * @c4 container
 *
 * Real-time trading dashboard with WebGL charts and streaming data.
 *
 * @c4 uses api_gateway "Market data" "WebSocket"
 * @c4 uses auth "Session tokens" "REST"
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
 * | `core.ts` | Mediator | Dashboard orchestration | stable |
 * | `state.ts` | Observer | Reactive state management | active |
 * | `config.ts` | -- | Dashboard configuration | stable |
 */
export { DashboardCore } from './core';
export { DashboardState } from './state';
export { Config } from './config';
