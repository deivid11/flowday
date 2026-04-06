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
