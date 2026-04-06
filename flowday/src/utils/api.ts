import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Block, PushResult, CalendarEvent, Conflict, SyncResult, GoogleCalendarEvent, TimerTickPayload, BlockCompletedPayload } from '../types';

// Block CRUD — param names must match Rust snake_case command params
export async function getBlocks(): Promise<Block[]> {
  return invoke<Block[]>('get_blocks');
}

export async function addBlock(block: Omit<Block, 'id'>): Promise<Block> {
  return invoke<Block>('add_block', { block });
}

export async function editBlock(block: Block): Promise<Block> {
  return invoke<Block>('edit_block', { block });
}

export async function deleteBlock(id: string): Promise<void> {
  return invoke('delete_block', { id });
}

export async function reorderBlocks(ids: string[]): Promise<void> {
  return invoke('reorder_blocks', { ids });
}

// Timer controls
export async function startTimer(blockId: string, durationSecs: number): Promise<void> {
  return invoke('start_timer', { blockId, durationSecs });
}

export async function pauseTimer(): Promise<void> {
  return invoke('pause_timer');
}

export async function resumeTimer(): Promise<void> {
  return invoke('resume_timer');
}

export async function stopTimer(): Promise<void> {
  return invoke('stop_timer');
}

export async function extendTimer(extraSecs: number): Promise<void> {
  return invoke('extend_timer', { extraSecs });
}

export async function getTimerState(): Promise<void> {
  return invoke('get_timer_state');
}

export async function togglePanel(expanded: boolean): Promise<void> {
  return invoke('toggle_panel', { expanded });
}

// Calendar push
export async function pushBlockToCalendar(blockId: string): Promise<PushResult> {
  return invoke<PushResult>('push_block_to_calendar', { blockId });
}

export async function unpushBlockFromCalendar(blockId: string): Promise<PushResult> {
  return invoke<PushResult>('unpush_block_from_calendar', { blockId });
}

export async function recordInterruption(blockId: string): Promise<void> {
  return invoke('record_interruption', { blockId });
}

// Calendar sync
export async function calendarSync(events: CalendarEvent[]): Promise<SyncResult> {
  return invoke<SyncResult>('calendar_sync', { events });
}

export async function getCalendarEvents(): Promise<CalendarEvent[]> {
  return invoke<CalendarEvent[]>('get_calendar_events');
}

export async function getConflicts(): Promise<Conflict[]> {
  return invoke<Conflict[]>('get_conflicts');
}

export async function getLastSyncTime(): Promise<string> {
  return invoke<string>('get_last_sync_time');
}

// Google Calendar API
export async function googleSetOauthConfig(clientId: string, clientSecret: string): Promise<void> {
  return invoke('google_set_oauth_config', { clientId, clientSecret });
}

export async function googleListAccounts(): Promise<string[]> {
  return invoke<string[]>('google_list_accounts');
}

export async function googleIsAuthenticated(email: string): Promise<boolean> {
  return invoke<boolean>('google_is_authenticated', { email });
}

export async function googleFetchEvents(email: string, startDate: string, endDate: string): Promise<GoogleCalendarEvent[]> {
  return invoke<GoogleCalendarEvent[]>('google_fetch_events', { email, startDate, endDate });
}

// Event listeners
export function onTimerTick(callback: (payload: TimerTickPayload) => void): Promise<UnlistenFn> {
  return listen<TimerTickPayload>('timer-tick', (event) => callback(event.payload));
}

export function onBlockCompleted(callback: (payload: BlockCompletedPayload) => void): Promise<UnlistenFn> {
  return listen<BlockCompletedPayload>('block-completed', (event) => callback(event.payload));
}
