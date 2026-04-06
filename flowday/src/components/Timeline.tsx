import { useState } from 'react';
import type { Block } from '../types';
import { BlockCard } from './BlockCard';

interface TimelineProps {
  blocks: Block[];
  activeBlockId: string | null;
  onEditBlock: (block: Block) => void;
  onDeleteBlock: (id: string) => void;
  onReorderBlocks: (ids: string[]) => void;
  onPushBlock: (blockId: string) => void;
  onUnpushBlock: (blockId: string) => void;
}

export function Timeline({ blocks, activeBlockId, onEditBlock, onDeleteBlock, onReorderBlocks, onPushBlock, onUnpushBlock }: TimelineProps) {
  const [dragOverId, setDragOverId] = useState<string | null>(null);
  const [draggedId, setDraggedId] = useState<string | null>(null);

  if (blocks.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-neutral-500 text-sm">
        No blocks scheduled. Use Quick Actions to start.
      </div>
    );
  }

  function handleDragStart(e: React.DragEvent, blockId: string) {
    setDraggedId(blockId);
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', blockId);
  }

  function handleDragOver(e: React.DragEvent, blockId: string) {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    setDragOverId(blockId);
  }

  function handleDrop(e: React.DragEvent, targetId: string) {
    e.preventDefault();
    const sourceId = e.dataTransfer.getData('text/plain');
    if (!sourceId || sourceId === targetId) {
      resetDragState();
      return;
    }

    const ids = blocks.map((b) => b.id);
    const fromIdx = ids.indexOf(sourceId);
    const toIdx = ids.indexOf(targetId);
    if (fromIdx === -1 || toIdx === -1) {
      resetDragState();
      return;
    }

    ids.splice(fromIdx, 1);
    ids.splice(toIdx, 0, sourceId);
    onReorderBlocks(ids);
    resetDragState();
  }

  function handleDragEnd() {
    resetDragState();
  }

  function resetDragState() {
    setDraggedId(null);
    setDragOverId(null);
  }

  return (
    <div className="flex-1 overflow-y-auto px-4 py-3 space-y-1.5" onDragEnd={handleDragEnd}>
      {blocks.map((block) => (
        <div
          key={block.id}
          className={`flex gap-3 transition-opacity ${
            draggedId === block.id ? 'opacity-40' : ''
          } ${dragOverId === block.id && draggedId !== block.id ? 'border-t-2 border-blue-400' : ''}`}
        >
          <div className="w-12 shrink-0 text-right text-xs text-neutral-500 pt-3 font-mono">
            {block.startTime}
          </div>
          <div className="flex-1">
            <BlockCard
              block={block}
              isActive={block.id === activeBlockId}
              onEdit={onEditBlock}
              onDelete={onDeleteBlock}
              onPush={onPushBlock}
              onUnpush={onUnpushBlock}
              draggable
              onDragStart={handleDragStart}
              onDragOver={(e) => handleDragOver(e, block.id)}
              onDrop={(e) => handleDrop(e, block.id)}
            />
          </div>
        </div>
      ))}
    </div>
  );
}
