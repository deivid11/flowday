import { useState } from 'react';
import type { Block } from '../types';
import { BLOCK_BG_CLASSES, BLOCK_TEXT_CLASSES, BLOCK_LABELS } from '../utils/constants';

interface BlockCardProps {
  block: Block;
  isActive: boolean;
  onEdit: (block: Block) => void;
  onDelete: (id: string) => void;
  onPush?: (blockId: string) => void;
  onUnpush?: (blockId: string) => void;
  draggable?: boolean;
  onDragStart?: (e: React.DragEvent, blockId: string) => void;
  onDragOver?: (e: React.DragEvent) => void;
  onDrop?: (e: React.DragEvent, blockId: string) => void;
}

export function BlockCard({ block, isActive, onEdit, onDelete, onPush, onUnpush, draggable, onDragStart, onDragOver, onDrop }: BlockCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [hovered, setHovered] = useState(false);
  const [confirmingPush, setConfirmingPush] = useState(false);

  const bgClass = BLOCK_BG_CLASSES[block.type];
  const textClass = BLOCK_TEXT_CLASSES[block.type];
  const canPush = block.type === 'DeepWork' && !block.pushedToCalendar;
  const isPushed = block.pushedToCalendar;

  function handlePushClick(e: React.MouseEvent) {
    e.stopPropagation();
    if (isPushed) {
      onUnpush?.(block.id);
    } else {
      setConfirmingPush(true);
    }
  }

  function handleConfirmPush(e: React.MouseEvent) {
    e.stopPropagation();
    setConfirmingPush(false);
    onPush?.(block.id);
  }

  function handleCancelPush(e: React.MouseEvent) {
    e.stopPropagation();
    setConfirmingPush(false);
  }

  return (
    <div
      className={`relative rounded-lg border p-3 cursor-pointer transition-all ${bgClass} ${isActive ? 'ring-2 ring-white/30 scale-[1.02]' : 'hover:brightness-110'}`}
      onClick={() => setExpanded(!expanded)}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => { setHovered(false); setConfirmingPush(false); }}
      draggable={draggable}
      onDragStart={(e) => onDragStart?.(e, block.id)}
      onDragOver={(e) => { e.preventDefault(); onDragOver?.(e); }}
      onDrop={(e) => onDrop?.(e, block.id)}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 min-w-0">
          {draggable && (
            <span className="text-neutral-500 cursor-grab active:cursor-grabbing text-xs select-none" title="Drag to reorder">⠿</span>
          )}
          <span className={`text-xs font-semibold uppercase tracking-wide ${textClass}`}>
            {BLOCK_LABELS[block.type]}
          </span>
          <span className="text-sm font-medium text-neutral-100 truncate">
            {block.name}
          </span>
          {isPushed && (
            <span className="text-green-400 text-xs" title="Pushed to calendar">✓ Cal</span>
          )}
        </div>
        <div className="flex items-center gap-2 shrink-0">
          <span className="text-xs text-neutral-400">{block.duration}m</span>
          {block.interruptionCount > 0 && (
            <span className="text-xs bg-red-500/30 text-red-300 px-1.5 py-0.5 rounded">
              {block.interruptionCount} int
            </span>
          )}
        </div>
      </div>

      {/* Push confirmation inline */}
      {confirmingPush && (
        <div className="mt-2 pt-2 border-t border-white/10 flex items-center gap-2">
          <span className="text-xs text-neutral-300">Push to calendar?</span>
          <button
            className="px-2 py-0.5 text-xs rounded bg-green-600 hover:bg-green-700 text-white transition-colors"
            onClick={handleConfirmPush}
          >
            Yes
          </button>
          <button
            className="px-2 py-0.5 text-xs rounded bg-neutral-600 hover:bg-neutral-500 text-neutral-200 transition-colors"
            onClick={handleCancelPush}
          >
            No
          </button>
        </div>
      )}

      {expanded && !confirmingPush && (
        <div className="mt-2 pt-2 border-t border-white/10 text-xs text-neutral-400 space-y-1">
          <div>Start: {block.startTime}</div>
          <div>Duration: {block.duration} min</div>
          {block.notes && <div>Notes: {block.notes}</div>}
          {block.pauseTime > 0 && <div>Paused: {block.pauseTime}s</div>}
          {isPushed && <div className="text-green-400">Pushed to calendar</div>}
        </div>
      )}

      {hovered && (
        <div className="absolute top-2 right-2 flex items-center gap-1">
          {(canPush || isPushed) && (
            <button
              className={`text-xs transition-colors p-0.5 ${isPushed ? 'text-green-400 hover:text-red-400' : 'text-neutral-500 hover:text-green-400'}`}
              onClick={handlePushClick}
              title={isPushed ? 'Remove from calendar' : 'Push to calendar'}
            >
              {isPushed ? '✓' : '📅'}
            </button>
          )}
          <button
            className="text-neutral-500 hover:text-blue-400 text-xs transition-colors p-0.5"
            onClick={(e) => { e.stopPropagation(); onEdit(block); }}
            title="Edit block"
          >
            ✎
          </button>
          <button
            className="text-neutral-500 hover:text-red-400 text-xs transition-colors p-0.5"
            onClick={(e) => { e.stopPropagation(); onDelete(block.id); }}
            title="Delete block"
          >
            ✕
          </button>
        </div>
      )}

      {isActive && (
        <div className="absolute -left-1 top-1/2 -translate-y-1/2 w-1 h-6 bg-white rounded-full" />
      )}
    </div>
  );
}
