import type { Block } from '../types';
import { useTimer } from '../hooks/useTimer';
import { BLOCK_COLORS } from '../utils/constants';
import * as api from '../utils/api';

interface QuickActionsProps {
  timer: ReturnType<typeof useTimer>;
  currentBlock: Block | null;
  onAddBlock: (block: Omit<Block, 'id'>) => void;
  onOpenAddForm?: () => void;
}

function nextStartTime(): string {
  const now = new Date();
  const m = now.getMinutes();
  // Round up to next 15-minute slot
  const rounded = Math.ceil(m / 15) * 15;
  now.setMinutes(rounded, 0, 0);
  if (rounded >= 60) now.setHours(now.getHours() + 1, 0, 0, 0);
  return `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
}

export function QuickActions({ timer, currentBlock, onAddBlock, onOpenAddForm }: QuickActionsProps) {
  const handleStartDeepWork = () => {
    const block: Omit<Block, 'id'> = {
      name: 'Deep Work',
      type: 'DeepWork',
      startTime: nextStartTime(),
      duration: 90,
      color: BLOCK_COLORS.DeepWork,
      pauseTime: 0,
      interruptionCount: 0,
    };
    onAddBlock(block);
  };

  const handleStartReactive = () => {
    const block: Omit<Block, 'id'> = {
      name: 'Reactive',
      type: 'Reactive',
      startTime: nextStartTime(),
      duration: 30,
      color: BLOCK_COLORS.Reactive,
      pauseTime: 0,
      interruptionCount: 0,
    };
    onAddBlock(block);
  };

  const handleInterruption = async () => {
    if (currentBlock) {
      try { await api.recordInterruption(currentBlock.id); } catch { /* local fallback handled in hooks */ }
    }
  };

  return (
    <div className="px-4 py-3 border-t border-neutral-700/50 space-y-2">
      {!timer.isRunning && (
        <div className="flex gap-2">
          <button
            onClick={handleStartDeepWork}
            className="flex-1 px-3 py-2 text-sm font-medium rounded-lg bg-blue-600 hover:bg-blue-700 text-white transition-colors"
          >
            Start Deep Work
          </button>
          <button
            onClick={handleStartReactive}
            className="flex-1 px-3 py-2 text-sm font-medium rounded-lg bg-orange-600 hover:bg-orange-700 text-white transition-colors"
          >
            Start Reactive
          </button>
        </div>
      )}
      <div className="flex gap-2">
        {onOpenAddForm && (
          <button
            onClick={onOpenAddForm}
            className="px-3 py-1.5 text-xs font-medium rounded-lg bg-neutral-700 hover:bg-neutral-600 text-neutral-200 transition-colors"
            title="Add custom block"
          >
            + Block
          </button>
        )}
        <button
          onClick={() => timer.extend(15)}
          disabled={!timer.isRunning}
          className="flex-1 px-3 py-1.5 text-xs font-medium rounded-lg bg-neutral-700 hover:bg-neutral-600 text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          +15 min
        </button>
        <button
          onClick={timer.skip}
          disabled={!timer.isRunning}
          className="flex-1 px-3 py-1.5 text-xs font-medium rounded-lg bg-neutral-700 hover:bg-neutral-600 text-neutral-200 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          Skip
        </button>
        <button
          onClick={handleInterruption}
          disabled={!timer.isRunning}
          className="flex-1 px-3 py-1.5 text-xs font-medium rounded-lg bg-red-900/50 hover:bg-red-800/50 text-red-300 disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
        >
          Interrupted
        </button>
      </div>
    </div>
  );
}
