import type { BlockType } from '../types';

export const BLOCK_COLORS: Record<BlockType, string> = {
  DeepWork: '#3b82f6',  // blue-500
  Reactive: '#f97316',  // orange-500
  Meeting: '#22c55e',   // green-500
  Admin: '#ef4444',     // red-500
  Break: '#6b7280',     // gray-500
};

export const BLOCK_BG_CLASSES: Record<BlockType, string> = {
  DeepWork: 'bg-blue-500/20 border-blue-500/40',
  Reactive: 'bg-orange-500/20 border-orange-500/40',
  Meeting: 'bg-green-500/20 border-green-500/40',
  Admin: 'bg-red-500/20 border-red-500/40',
  Break: 'bg-gray-500/20 border-gray-500/40',
};

export const BLOCK_TEXT_CLASSES: Record<BlockType, string> = {
  DeepWork: 'text-blue-400',
  Reactive: 'text-orange-400',
  Meeting: 'text-green-400',
  Admin: 'text-red-400',
  Break: 'text-gray-400',
};

export const BLOCK_LABELS: Record<BlockType, string> = {
  DeepWork: 'Deep Work',
  Reactive: 'Reactive',
  Meeting: 'Meeting',
  Admin: 'Admin',
  Break: 'Break',
};

export const DEFAULT_DURATIONS: Partial<Record<BlockType, number>> = {
  DeepWork: 90,
  Reactive: 30,
};
