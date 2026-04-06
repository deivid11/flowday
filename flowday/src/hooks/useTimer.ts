import { useState, useEffect, useCallback, useRef } from 'react';
import type { TimerState } from '../types';
import * as api from '../utils/api';

export function useTimer() {
  const [state, setState] = useState<TimerState>({
    isRunning: false,
    activeBlockId: null,
    remainingSeconds: 0,
  });

  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Local countdown every second when running
  useEffect(() => {
    if (state.isRunning && state.remainingSeconds > 0) {
      intervalRef.current = setInterval(() => {
        setState((prev) => {
          if (prev.remainingSeconds <= 1) {
            return { ...prev, remainingSeconds: 0, isRunning: false, activeBlockId: null };
          }
          return { ...prev, remainingSeconds: prev.remainingSeconds - 1 };
        });
      }, 1000);
    }
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [state.isRunning, state.remainingSeconds > 0]);

  // Listen to Tauri timer-tick events for sync
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    api.onTimerTick((payload) => {
      setState((prev) => ({
        ...prev,
        remainingSeconds: payload.remaining_seconds,
        activeBlockId: payload.block_id,
        isRunning: true,
      }));
    }).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  }, []);

  // Listen to block-completed events
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    api.onBlockCompleted(() => {
      setState({ isRunning: false, activeBlockId: null, remainingSeconds: 0 });
    }).then((fn) => { unlisten = fn; });

    return () => { unlisten?.(); };
  }, []);

  const start = useCallback(async (blockId: string, durationMinutes: number) => {
    const durationSecs = durationMinutes * 60;
    try {
      await api.startTimer(blockId, durationSecs);
    } catch { /* local fallback */ }
    setState({ isRunning: true, activeBlockId: blockId, remainingSeconds: durationSecs });
  }, []);

  const pause = useCallback(async () => {
    try { await api.pauseTimer(); } catch { /* local fallback */ }
    setState((prev) => ({ ...prev, isRunning: false }));
  }, []);

  const resume = useCallback(async () => {
    try { await api.resumeTimer(); } catch { /* local fallback */ }
    setState((prev) => ({ ...prev, isRunning: true }));
  }, []);

  const skip = useCallback(async () => {
    try { await api.stopTimer(); } catch { /* local fallback */ }
    setState({ isRunning: false, activeBlockId: null, remainingSeconds: 0 });
  }, []);

  const extend = useCallback(async (minutes: number = 15) => {
    const extraSecs = minutes * 60;
    try { await api.extendTimer(extraSecs); } catch { /* local fallback */ }
    setState((prev) => ({
      ...prev,
      remainingSeconds: prev.remainingSeconds + extraSecs,
    }));
  }, []);

  const formatTime = useCallback((totalSeconds: number) => {
    const m = Math.floor(totalSeconds / 60);
    const s = totalSeconds % 60;
    return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  }, []);

  return {
    ...state,
    start,
    pause,
    resume,
    skip,
    extend,
    formattedTime: formatTime(state.remainingSeconds),
  };
}
