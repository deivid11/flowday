export type BlockType = 'DeepWork' | 'Reactive' | 'Meeting' | 'Admin' | 'Break';

export interface Block {
  id: string;
  name: string;
  type: BlockType;
  startTime: string; // HH:MM format
  duration: number; // minutes
  color: string;
  notes?: string;
  pauseTime: number;
  interruptionCount: number;
  pushedToCalendar: boolean;
  calendarEventId?: string | null;
}

export interface PushResult {
  blockId: string;
  calendarEventId: string;
  pushed: boolean;
}

export interface TimerState {
  isRunning: boolean;
  activeBlockId: string | null;
  remainingSeconds: number;
}

export interface TimerTickPayload {
  remaining_seconds: number;
  block_id: string;
}

export interface BlockCompletedPayload {
  block_id: string;
}

// Calendar sync types
export interface CalendarEvent {
  id: string;
  googleEventId: string;
  summary: string;
  startTime: string; // HH:MM
  endTime: string;   // HH:MM
  date: string;      // YYYY-MM-DD
  allDay: boolean;
  status: string;
}

export interface Conflict {
  blockId: string;
  blockName: string;
  blockStart: string;
  blockEnd: string;
  eventId: string;
  eventSummary: string;
  eventStart: string;
  eventEnd: string;
  overlapMinutes: number;
}

export interface SyncResult {
  eventsSynced: number;
  conflicts: Conflict[];
  lastSyncedAt: string;
}

// Google Calendar API event (raw from google_fetch_events)
export interface GoogleCalendarEvent {
  id: string | null;
  summary: string;
  start: { dateTime?: string; date?: string; timeZone?: string };
  end: { dateTime?: string; date?: string; timeZone?: string };
  description?: string;
  status?: string;
}
