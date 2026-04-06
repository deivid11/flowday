import { useState, useCallback } from 'react';
import type { Block } from '../types';
import { MenuBar } from './MenuBar';
import { Timeline } from './Timeline';
import { QuickActions } from './QuickActions';
import { BlockFormModal } from './BlockFormModal';
import { DeleteConfirmDialog } from './DeleteConfirmDialog';
import { useTimer } from '../hooks/useTimer';
import { useBlocks } from '../hooks/useBlocks';
import * as api from '../utils/api';

export function DropdownPanel() {
  const timer = useTimer();
  const { blocks, currentBlock, upcomingBlock, addBlock, editBlock, deleteBlock, reorderBlocks, isCreating, isUpdating, isDeleting } = useBlocks();
  const [isPanelOpen, setIsPanelOpen] = useState(true);

  // Modal state
  const [isFormOpen, setIsFormOpen] = useState(false);
  const [editingBlock, setEditingBlock] = useState<Block | null>(null);
  const [deletingBlock, setDeletingBlock] = useState<Block | null>(null);

  const togglePanel = useCallback(() => {
    const next = !isPanelOpen;
    setIsPanelOpen(next);
    api.togglePanel(next).catch(() => { /* not in Tauri env */ });
  }, [isPanelOpen]);

  function handleOpenAddForm() {
    setEditingBlock(null);
    setIsFormOpen(true);
  }

  function handleEditBlock(block: Block) {
    setEditingBlock(block);
    setIsFormOpen(true);
  }

  function handleDeleteRequest(id: string) {
    const block = blocks.find((b) => b.id === id);
    if (block) setDeletingBlock(block);
  }

  async function handleFormSave(data: Omit<Block, 'id'> | Block) {
    if ('id' in data) {
      await editBlock(data as Block);
    } else {
      await addBlock(data);
    }
    setIsFormOpen(false);
    setEditingBlock(null);
  }

  async function handleDeleteConfirm() {
    if (!deletingBlock) return;
    await deleteBlock(deletingBlock.id);
    setDeletingBlock(null);
  }

  return (
    <div className="w-[420px] flex flex-col bg-neutral-900 text-white rounded-xl overflow-hidden border border-neutral-700/50 shadow-2xl">
      {/* Menu Bar - fixed 60px */}
      <MenuBar timer={timer} upcomingBlock={upcomingBlock} onTogglePanel={togglePanel} />

      {/* Collapsible body with animation */}
      <div
        className="transition-all duration-200 ease-in-out overflow-hidden"
        style={{
          maxHeight: isPanelOpen ? '540px' : '0px',
          opacity: isPanelOpen ? 1 : 0,
        }}
      >
        {/* Timeline - fills middle, scrollable */}
        <div className="h-[480px] overflow-y-auto">
          <Timeline
            blocks={blocks}
            activeBlockId={timer.activeBlockId}
            onEditBlock={handleEditBlock}
            onDeleteBlock={handleDeleteRequest}
            onReorderBlocks={reorderBlocks}
          />
        </div>

        {/* Quick Actions - fixed 60px */}
        <div className="h-[60px] flex-shrink-0">
          <QuickActions
            timer={timer}
            currentBlock={currentBlock}
            onAddBlock={addBlock}
            onOpenAddForm={handleOpenAddForm}
          />
        </div>
      </div>

      {isFormOpen && (
        <BlockFormModal
          block={editingBlock}
          onSave={handleFormSave}
          onClose={() => { setIsFormOpen(false); setEditingBlock(null); }}
          isSaving={isCreating || isUpdating}
        />
      )}

      {deletingBlock && (
        <DeleteConfirmDialog
          blockName={deletingBlock.name}
          onConfirm={handleDeleteConfirm}
          onCancel={() => setDeletingBlock(null)}
          isDeleting={isDeleting}
        />
      )}
    </div>
  );
}
