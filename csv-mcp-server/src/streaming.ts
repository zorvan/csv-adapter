/**
 * Streaming utilities for CSV MCP Server v2
 *
 * Provides the TransferProgressStream class that emits typed progress events
 * over SSE, with support for pause / resume / cancel and elapsed-time tracking.
 */

import {
  StreamStatus,
  TransferProgressEvent,
  StreamingOptions,
  StreamState,
} from "../types/streaming.js";

// ---------------------------------------------------------------------------
// Callback signatures
// ---------------------------------------------------------------------------

export type OnProgressCallback = (event: TransferProgressEvent) => void;
export type OnStatusChangeCallback = (status: StreamStatus) => void;

// ---------------------------------------------------------------------------
// TransferProgressStream
// ---------------------------------------------------------------------------

export class TransferProgressStream {
  private _state: StreamState;
  private _options: Required<StreamingOptions>;
  private _listeners: Set<OnProgressCallback> = new Set();
  private _statusListeners: Set<OnStatusChangeCallback> = new Set();
  private _lastEmitMs = 0;
  private _timeoutTimer: ReturnType<typeof setTimeout> | null = null;

  constructor(options?: StreamingOptions) {
    this._options = {
      stream: options?.stream ?? false,
      throttle_ms: options?.throttle_ms ?? 500,
      timeout_ms: options?.timeout_ms ?? 600_000, // 10 minutes
    };

    this._state = {
      status: StreamStatus.IDLE,
      startTime: null,
      elapsedTimeMs: 0,
      lastEvent: null,
      error: null,
    };
  }

  // ---- Public API ----------------------------------------------------------

  /** Whether streaming is enabled for this instance */
  get isStreamingEnabled(): boolean {
    return this._options.stream;
  }

  /** Current stream status */
  get status(): StreamStatus {
    return this._state.status;
  }

  /** Total elapsed time in milliseconds */
  get elapsedMs(): number {
    if (this._state.startTime === null) return this._state.elapsedTimeMs;
    return this._state.elapsedTimeMs + (Date.now() - this._state.startTime);
  }

  /** The last emitted progress event (or null) */
  get lastEvent(): TransferProgressEvent | null {
    return this._state.lastEvent;
  }

  // ---- Lifecycle -----------------------------------------------------------

  /** Mark the stream as started; begins the timeout countdown */
  start(): void {
    if (this._state.status !== StreamStatus.IDLE) return;

    this._state.status = StreamStatus.RUNNING;
    this._state.startTime = Date.now();
    this._notifyStatusChange(StreamStatus.RUNNING);

    // Set global timeout
    this._timeoutTimer = setTimeout(() => {
      this.fail(new Error(`Transfer timed out after ${this._options.timeout_ms}ms`));
    }, this._options.timeout_ms);
  }

  /** Pause the stream — no further events will be emitted until resumed */
  pause(): void {
    if (this._state.status !== StreamStatus.RUNNING) return;
    this._state.status = StreamStatus.PAUSED;
    this._notifyStatusChange(StreamStatus.PAUSED);
  }

  /** Resume a previously paused stream */
  resume(): void {
    if (this._state.status !== StreamStatus.PAUSED) return;
    this._state.status = StreamStatus.RUNNING;
    this._notifyStatusChange(StreamStatus.RUNNING);
  }

  /** Cancel the stream — no further events, final status is CANCELLED */
  cancel(): void {
    this._clearState();
    this._state.status = StreamStatus.CANCELLED;
    this._finalize();
  }

  /** Mark the stream as completed successfully */
  complete(): void {
    this._clearState();
    this._state.status = StreamStatus.COMPLETED;
    this._finalize();
  }

  /** Mark the stream as failed with an error */
  fail(error: Error): void {
    this._clearState();
    this._state.status = StreamStatus.ERROR;
    this._state.error = error;
    this._finalize();
  }

  // ---- Event emission ------------------------------------------------------

  /**
   * Emit a progress event to all registered listeners.
   *
   * Respects the throttle interval configured in options.
   * Returns false if the event was throttled (not emitted).
   */
  emit(event: Omit<TransferProgressEvent, "timestamp">): boolean {
    // Only emit when running
    if (this._state.status !== StreamStatus.RUNNING) return false;

    // Throttle check
    const now = Date.now();
    if (now - this._lastEmitMs < this._options.throttle_ms) {
      return false;
    }

    const fullEvent: TransferProgressEvent = {
      ...event,
      timestamp: new Date().toISOString(),
    };

    this._lastEmitMs = now;
    this._state.lastEvent = fullEvent;
    this._state.elapsedTimeMs = now - (this._state.startTime ?? now);

    for (const listener of this._listeners) {
      try {
        listener(fullEvent);
      } catch (err) {
        console.error("[TransferProgressStream] Listener error:", err);
      }
    }

    return true;
  }

  // ---- Listener management -------------------------------------------------

  onProgress(callback: OnProgressCallback): () => void {
    this._listeners.add(callback);
    return () => {
      this._listeners.delete(callback);
    };
  }

  onStatusChange(callback: OnStatusChangeCallback): () => void {
    this._statusListeners.add(callback);
    return () => {
      this._statusListeners.delete(callback);
    };
  }

  // ---- Internals -----------------------------------------------------------

  private _clearState(): void {
    if (this._timeoutTimer !== null) {
      clearTimeout(this._timeoutTimer);
      this._timeoutTimer = null;
    }
  }

  private _finalize(): void {
    if (this._state.startTime !== null) {
      this._state.elapsedTimeMs = Date.now() - this._state.startTime;
    }
    this._clearState();
  }

  private _notifyStatusChange(status: StreamStatus): void {
    for (const listener of this._statusListeners) {
      try {
        listener(status);
      } catch (err) {
        console.error("[TransferProgressStream] Status listener error:", err);
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Estimate remaining seconds based on current progress and elapsed time */
export function estimateRemaining(
  progressPercent: number,
  elapsedMs: number
): number | null {
  if (progressPercent <= 0 || progressPercent >= 100) return null;
  const totalEstimated = elapsedMs / (progressPercent / 100);
  return Math.round((totalEstimated - elapsedMs) / 1000);
}
