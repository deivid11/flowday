import { useTimer } from '../hooks/useTimer';
import type { Block } from '../types';

interface MenuBarProps {
  timer: ReturnType<typeof useTimer>;
  upcomingBlock: Block | null;
  onTogglePanel: () => void;
}

function formatHHMM(totalSeconds: number): string {
  const h = Math.floor(totalSeconds / 3600);
  const m = Math.floor((totalSeconds % 3600) / 60);
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}`;
}

export function MenuBar({ timer, upcomingBlock, onTogglePanel }: MenuBarProps) {
  return (
    <button
      onClick={onTogglePanel}
      className="w-full flex items-center justify-between px-4 h-[60px] bg-neutral-800/80 border-b border-neutral-700/50 cursor-pointer hover:bg-neutral-750/90 transition-colors select-none"
    >
      <span className="text-xs font-semibold text-neutral-400 uppercase tracking-wider">
        FlowDay
      </span>
      <div className="text-sm font-mono">
        {timer.isRunning ? (
          <span className="text-blue-400 animate-pulse">
            ⏱ {formatHHMM(timer.remainingSeconds)}
          </span>
        ) : upcomingBlock ? (
          <span className="text-neutral-300">
            Next: {upcomingBlock.startTime}
          </span>
        ) : (
          <span className="text-neutral-500">No blocks</span>
        )}
      </div>
    </button>
  );
}
