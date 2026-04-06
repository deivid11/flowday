import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { Block, TimerTickPayload, BlockCompletedPayload } from '../types';

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

export async function recordInterruption(blockId: string): Promise<void> {
  return invoke('record_interruption', { blockId });
}

// Event listeners
export function onTimerTick(callback: (payload: TimerTickPayload) => void): Promise<UnlistenFn> {
  return listen<TimerTickPayload>('timer-tick', (event) => callback(event.payload));
}

export function onBlockCompleted(callback: (payload: BlockCompletedPayload) => void): Promise<UnlistenFn> {
  return listen<BlockCompletedPayload>('block-completed', (event) => callback(event.payload));
}
