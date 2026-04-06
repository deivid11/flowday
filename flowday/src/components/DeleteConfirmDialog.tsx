interface DeleteConfirmDialogProps {
  blockName: string;
  onConfirm: () => void;
  onCancel: () => void;
  isDeleting?: boolean;
}

export function DeleteConfirmDialog({ blockName, onConfirm, onCancel, isDeleting }: DeleteConfirmDialogProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onCancel}>
      <div
        className="w-[340px] bg-neutral-800 border border-neutral-600/50 rounded-xl shadow-2xl p-5"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-lg font-semibold text-neutral-100 mb-2">Delete Block</h2>
        <p className="text-sm text-neutral-400 mb-5">
          Are you sure you want to delete <span className="text-neutral-200 font-medium">"{blockName}"</span>? This cannot be undone.
        </p>
        <div className="flex justify-end gap-2">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm text-neutral-400 hover:text-neutral-200 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            disabled={isDeleting}
            className="px-4 py-2 text-sm bg-red-600 hover:bg-red-500 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {isDeleting ? 'Deleting...' : 'Delete'}
          </button>
        </div>
      </div>
    </div>
  );
}
