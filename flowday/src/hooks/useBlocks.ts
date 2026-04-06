import { useState, useEffect, useCallback } from 'react';
import type { Block, CalendarEvent, Conflict } from '../types';
import * as api from '../utils/api';
import { BLOCK_COLORS } from '../utils/constants';
import { calendarSync, type CalendarSyncState, type SyncStatus } from '../utils/calendarSync';

// Demo blocks used when Tauri backend is unavailable (browser dev mode)
const DEMO_BLOCKS: Block[] = [
  { id: '1', name: 'Morning Focus', type: 'DeepWork', startTime: '08:00', duration: 90, color: BLOCK_COLORS.DeepWork, pauseTime: 0, interruptionCount: 0, pushedToCalendar: false, calendarEventId: null },
  { id: '2', name: 'Email & Slack', type: 'Reactive', startTime: '09:30', duration: 30, color: BLOCK_COLORS.Reactive, pauseTime: 0, interruptionCount: 0, pushedToCalendar: false, calendarEventId: null },
  { id: '3', name: 'Team Standup', type: 'Meeting', startTime: '10:00', duration: 30, color: BLOCK_COLORS.Meeting, pauseTime: 0, interruptionCount: 0, pushedToCalendar: false, calendarEventId: null },
  { id: '4', name: 'Feature Dev', type: 'DeepWork', startTime: '10:30', duration: 90, color: BLOCK_COLORS.DeepWork, pauseTime: 0, interruptionCount: 1, pushedToCalendar: false, calendarEventId: null },
  { id: '5', name: 'Lunch Break', type: 'Break', startTime: '12:00', duration: 60, color: BLOCK_COLORS.Break, pauseTime: 0, interruptionCount: 0, pushedToCalendar: false, calendarEventId: null },
  { id: '6', name: 'Code Review', type: 'Admin', startTime: '13:00', duration: 45, color: BLOCK_COLORS.Admin, pauseTime: 0, interruptionCount: 0, pushedToCalendar: false, calendarEventId: null },
];

export function useBlocks() {
  const [blocks, setBlocks] = useState<Block[]>([]);
  const [loading, setLoading] = useState(true);
  const [usingBackend, setUsingBackend] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Calendar sync state
  const [calendarEvents, setCalendarEvents] = useState<CalendarEvent[]>([]);
  const [conflicts, setConflicts] = useState<Conflict[]>([]);
  const [syncStatus, setSyncStatus] = useState<SyncStatus>('idle');
  const [lastSyncedAt, setLastSyncedAt] = useState<string | null>(null);
  const [syncError, setSyncError] = useState<string | null>(null);

  const clearError = useCallback(() => setError(null), []);

  const fetchBlocks = useCallback(async () => {
    try {
      const result = await api.getBlocks();
      setBlocks(result);
      setUsingBackend(true);
    } catch {
      // Backend not ready — use demo data
      if (!usingBackend) {
        setBlocks(DEMO_BLOCKS);
      }
    } finally {
      setLoading(false);
    }
  }, [usingBackend]);

  useEffect(() => {
    fetchBlocks();
  }, [fetchBlocks]);

  // Listen for block-completed to refetch
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    api.onBlockCompleted(() => { fetchBlocks(); })
      .then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, [fetchBlocks]);

  const currentBlock = blocks.find((b) => {
    const now = new Date();
    const [h, m] = b.startTime.split(':').map(Number);
    const start = h * 60 + m;
    const end = start + b.duration;
    const current = now.getHours() * 60 + now.getMinutes();
    return current >= start && current < end;
  }) ?? null;

  const upcomingBlock = blocks.find((b) => {
    const now = new Date();
    const [h, m] = b.startTime.split(':').map(Number);
    const start = h * 60 + m;
    const current = now.getHours() * 60 + now.getMinutes();
    return start > current;
  }) ?? null;

  const addBlock = useCallback(async (block: Omit<Block, 'id'>) => {
    setIsCreating(true);
    setError(null);
    try {
      const created = await api.addBlock(block);
      setBlocks((prev) => [...prev, created]);
      setUsingBackend(true);
    } catch {
      // Local fallback
      const newBlock: Block = { ...block, id: crypto.randomUUID(), pushedToCalendar: false, calendarEventId: null };
      setBlocks((prev) => [...prev, newBlock]);
    } finally {
      setIsCreating(false);
    }
  }, []);

  const editBlock = useCallback(async (block: Block) => {
    setIsUpdating(true);
    setError(null);
    try {
      const updated = await api.editBlock(block);
      setBlocks((prev) => prev.map((b) => (b.id === updated.id ? updated : b)));
    } catch {
      setBlocks((prev) => prev.map((b) => (b.id === block.id ? block : b)));
    } finally {
      setIsUpdating(false);
    }
  }, []);

  const deleteBlock = useCallback(async (id: string) => {
    setIsDeleting(true);
    setError(null);
    try {
      await api.deleteBlock(id);
    } catch { /* local fallback */ }
    setBlocks((prev) => prev.filter((b) => b.id !== id));
    setIsDeleting(false);
  }, []);

  const pushBlock = useCallback(async (blockId: string) => {
    try {
      const result = await api.pushBlockToCalendar(blockId);
      setBlocks((prev) =>
        prev.map((b) =>
          b.id === blockId
            ? { ...b, pushedToCalendar: true, calendarEventId: result.calendarEventId }
            : b
        )
      );
    } catch (e) {
      setError(typeof e === 'string' ? e : 'Failed to push to calendar');
    }
  }, []);

  const unpushBlock = useCallback(async (blockId: string) => {
    try {
      await api.unpushBlockFromCalendar(blockId);
      setBlocks((prev) =>
        prev.map((b) =>
          b.id === blockId
            ? { ...b, pushedToCalendar: false, calendarEventId: null }
            : b
        )
      );
    } catch (e) {
      setError(typeof e === 'string' ? e : 'Failed to remove from calendar');
    }
  }, []);

  // Calendar sync lifecycle
  useEffect(() => {
    const unsub = calendarSync.subscribe((state: CalendarSyncState) => {
      setCalendarEvents(state.calendarEvents);
      setConflicts(state.conflicts);
      setSyncStatus(state.status);
      setLastSyncedAt(state.lastSyncedAt);
      setSyncError(state.error);
    });
    calendarSync.start();
    return () => {
      unsub();
      calendarSync.stop();
    };
  }, []);

  // Re-sync conflicts when blocks change (add/edit/delete may create or resolve conflicts)
  useEffect(() => {
    if (usingBackend && blocks.length > 0) {
      api.getConflicts().then(setConflicts).catch(() => {});
    }
  }, [blocks, usingBackend]);

  const forceCalendarSync = useCallback(async () => {
    await calendarSync.forceSync();
  }, []);

  /** Check if a specific block has any conflicts. */
  const getBlockConflicts = useCallback((blockId: string): Conflict[] => {
    return conflicts.filter((c) => c.blockId === blockId);
  }, [conflicts]);

  const reorderBlocks = useCallback(async (ids: string[]) => {
    const reordered = ids.map((id) => blocks.find((b) => b.id === id)).filter(Boolean) as Block[];
    setBlocks(reordered);
    try { await api.reorderBlocks(ids); } catch { /* keep local order */ }
  }, [blocks]);

  return {
    blocks,
    loading,
    currentBlock,
    upcomingBlock,
    addBlock,
    editBlock,
    deleteBlock,
    reorderBlocks,
    pushBlock,
    unpushBlock,
    refetch: fetchBlocks,
    isCreating,
    isUpdating,
    isDeleting,
    error,
    clearError,
    // Calendar sync
    calendarEvents,
    conflicts,
    syncStatus,
    lastSyncedAt,
    syncError,
    forceCalendarSync,
    getBlockConflicts,
  };
}
