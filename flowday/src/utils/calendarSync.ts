import type { CalendarEvent, GoogleCalendarEvent, SyncResult, Conflict } from '../types';
import * as api from './api';

const SYNC_INTERVAL_MS = 5 * 60 * 1000; // 5 minutes

/** Convert a Google Calendar API event to our local CalendarEvent format. */
function convertGoogleEvent(gEvent: GoogleCalendarEvent, date: string): CalendarEvent | null {
  const id = gEvent.id ?? crypto.randomUUID();

  // All-day events
  if (gEvent.start.date) {
    return {
      id,
      googleEventId: gEvent.id ?? id,
      summary: gEvent.summary,
      startTime: '00:00',
      endTime: '23:59',
      date,
      allDay: true,
      status: gEvent.status ?? 'confirmed',
    };
  }

  // Timed events — extract HH:MM from RFC3339 dateTime
  const startDt = gEvent.start.dateTime;
  const endDt = gEvent.end.dateTime;
  if (!startDt || !endDt) return null;

  const startTime = extractHHMM(startDt);
  const endTime = extractHHMM(endDt);
  if (!startTime || !endTime) return null;

  return {
    id,
    googleEventId: gEvent.id ?? id,
    summary: gEvent.summary,
    startTime,
    endTime,
    date,
    allDay: false,
    status: gEvent.status ?? 'confirmed',
  };
}

/** Extract HH:MM from an ISO/RFC3339 datetime string. */
function extractHHMM(dateTime: string): string | null {
  // Handles "2026-04-06T10:00:00Z", "2026-04-06T10:00:00+02:00", etc.
  const match = dateTime.match(/T(\d{2}):(\d{2})/);
  if (!match) return null;
  return `${match[1]}:${match[2]}`;
}

function todayISO(): string {
  const d = new Date();
  return d.toISOString().slice(0, 10);
}

export type SyncStatus = 'idle' | 'syncing' | 'error';

export interface CalendarSyncState {
  status: SyncStatus;
  lastSyncedAt: string | null;
  calendarEvents: CalendarEvent[];
  conflicts: Conflict[];
  error: string | null;
}

type SyncListener = (state: CalendarSyncState) => void;

/**
 * CalendarSyncManager handles:
 * - 5-minute background sync interval
 * - Sync-on-app-focus
 * - Google Calendar event fetching and conversion
 * - Caching events via Tauri backend
 * - Conflict detection
 */
class CalendarSyncManager {
  private state: CalendarSyncState = {
    status: 'idle',
    lastSyncedAt: null,
    calendarEvents: [],
    conflicts: [],
    error: null,
  };

  private listeners: Set<SyncListener> = new Set();
  private intervalId: ReturnType<typeof setInterval> | null = null;
  private focusHandler: (() => void) | null = null;

  /** Subscribe to state changes. Returns unsubscribe function. */
  subscribe(listener: SyncListener): () => void {
    this.listeners.add(listener);
    listener(this.state);
    return () => { this.listeners.delete(listener); };
  }

  private notify() {
    for (const listener of this.listeners) {
      listener(this.state);
    }
  }

  private updateState(partial: Partial<CalendarSyncState>) {
    this.state = { ...this.state, ...partial };
    this.notify();
  }

  /** Start the background sync loop and focus listener. */
  start() {
    // Initial sync
    this.performSync();

    // 5-minute interval
    this.intervalId = setInterval(() => {
      this.performSync();
    }, SYNC_INTERVAL_MS);

    // Sync on window focus
    this.focusHandler = () => { this.syncOnFocus(); };
    window.addEventListener('focus', this.focusHandler);
  }

  /** Stop the background sync loop and cleanup. */
  stop() {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
    if (this.focusHandler) {
      window.removeEventListener('focus', this.focusHandler);
      this.focusHandler = null;
    }
  }

  /** Sync on app focus — debounced to avoid rapid re-syncs. */
  private lastFocusSyncAt = 0;
  private syncOnFocus() {
    const now = Date.now();
    // Only sync on focus if at least 60 seconds since last sync
    if (now - this.lastFocusSyncAt < 60_000) return;
    this.lastFocusSyncAt = now;
    this.performSync();
  }

  /** Perform a full sync: fetch from Google, convert, cache, detect conflicts. */
  async performSync() {
    if (this.state.status === 'syncing') return;

    this.updateState({ status: 'syncing', error: null });

    try {
      const today = todayISO();

      // Step 1: Get authenticated Google accounts
      let accounts: string[] = [];
      try {
        accounts = await api.googleListAccounts();
      } catch {
        // Google not configured — load cached events and conflicts only
        await this.loadCachedState();
        return;
      }

      if (accounts.length === 0) {
        // No accounts linked — still check for cached data
        await this.loadCachedState();
        return;
      }

      // Step 2: Fetch events from all linked accounts
      const allConverted: CalendarEvent[] = [];

      for (const email of accounts) {
        try {
          const isAuth = await api.googleIsAuthenticated(email);
          if (!isAuth) continue;

          const rawEvents = await api.googleFetchEvents(email, today, today);
          for (const gEvent of rawEvents) {
            const converted = convertGoogleEvent(gEvent, today);
            if (converted) {
              allConverted.push(converted);
            }
          }
        } catch {
          // Skip this account on error, continue with others
        }
      }

      // Step 3: Push to backend cache and get conflict detection
      let syncResult: SyncResult;
      try {
        syncResult = await api.calendarSync(allConverted);
      } catch {
        // Backend not available — do local conflict detection only
        this.updateState({
          status: 'idle',
          calendarEvents: allConverted,
          conflicts: [],
          lastSyncedAt: new Date().toISOString(),
        });
        return;
      }

      this.updateState({
        status: 'idle',
        calendarEvents: allConverted,
        conflicts: syncResult.conflicts,
        lastSyncedAt: syncResult.lastSyncedAt,
      });

    } catch (err) {
      this.updateState({
        status: 'error',
        error: err instanceof Error ? err.message : String(err),
      });
    }
  }

  /** Load cached calendar events and conflicts from backend. */
  private async loadCachedState() {
    try {
      const [events, conflicts, lastSync] = await Promise.all([
        api.getCalendarEvents(),
        api.getConflicts(),
        api.getLastSyncTime(),
      ]);
      this.updateState({
        status: 'idle',
        calendarEvents: events,
        conflicts,
        lastSyncedAt: lastSync || null,
      });
    } catch {
      this.updateState({ status: 'idle' });
    }
  }

  /** Force an immediate sync. */
  async forceSync() {
    await this.performSync();
  }

  /** Get current state snapshot. */
  getState(): CalendarSyncState {
    return { ...this.state };
  }
}

// Singleton instance
export const calendarSync = new CalendarSyncManager();
