import { useState, useEffect } from 'react';
import type { Block, BlockType } from '../types';
import { BLOCK_COLORS, BLOCK_LABELS } from '../utils/constants';

const BLOCK_TYPES: BlockType[] = ['DeepWork', 'Reactive', 'Meeting', 'Admin', 'Break'];

interface BlockFormModalProps {
  block?: Block | null;
  onSave: (block: Omit<Block, 'id'> | Block) => void;
  onClose: () => void;
  isSaving?: boolean;
}

interface FormErrors {
  name?: string;
  duration?: string;
  startTime?: string;
}

export function BlockFormModal({ block, onSave, onClose, isSaving }: BlockFormModalProps) {
  const isEditing = !!block;
  const [name, setName] = useState(block?.name ?? '');
  const [type, setType] = useState<BlockType>(block?.type ?? 'DeepWork');
  const [startTime, setStartTime] = useState(block?.startTime ?? '09:00');
  const [duration, setDuration] = useState(block?.duration ?? 90);
  const [notes, setNotes] = useState(block?.notes ?? '');
  const [errors, setErrors] = useState<FormErrors>({});

  useEffect(() => {
    if (block) {
      setName(block.name);
      setType(block.type);
      setStartTime(block.startTime);
      setDuration(block.duration);
      setNotes(block.notes ?? '');
    }
  }, [block]);

  function validate(): boolean {
    const newErrors: FormErrors = {};
    if (!name.trim()) newErrors.name = 'Name is required';
    if (duration < 1 || duration > 480) newErrors.duration = 'Duration must be 1-480 minutes';
    if (!/^\d{2}:\d{2}$/.test(startTime)) {
      newErrors.startTime = 'Must be HH:MM format';
    } else {
      const [h, m] = startTime.split(':').map(Number);
      if (h < 0 || h > 23 || m < 0 || m > 59) newErrors.startTime = 'Invalid time';
    }
    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!validate()) return;

    const data = {
      name: name.trim(),
      type,
      startTime,
      duration,
      color: BLOCK_COLORS[type],
      notes: notes.trim() || undefined,
      pauseTime: block?.pauseTime ?? 0,
      interruptionCount: block?.interruptionCount ?? 0,
    };

    if (isEditing && block) {
      onSave({ ...data, id: block.id });
    } else {
      onSave(data);
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onClose}>
      <div
        className="w-[380px] bg-neutral-800 border border-neutral-600/50 rounded-xl shadow-2xl p-5"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-lg font-semibold text-neutral-100 mb-4">
          {isEditing ? 'Edit Block' : 'Add Block'}
        </h2>

        <form onSubmit={handleSubmit} className="space-y-3">
          {/* Name */}
          <div>
            <label className="block text-xs text-neutral-400 mb-1">Name</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full bg-neutral-700 border border-neutral-600 rounded-lg px-3 py-2 text-sm text-neutral-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
              placeholder="Block name"
              autoFocus
            />
            {errors.name && <p className="text-xs text-red-400 mt-1">{errors.name}</p>}
          </div>

          {/* Type */}
          <div>
            <label className="block text-xs text-neutral-400 mb-1">Type</label>
            <select
              value={type}
              onChange={(e) => setType(e.target.value as BlockType)}
              className="w-full bg-neutral-700 border border-neutral-600 rounded-lg px-3 py-2 text-sm text-neutral-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
            >
              {BLOCK_TYPES.map((t) => (
                <option key={t} value={t}>{BLOCK_LABELS[t]}</option>
              ))}
            </select>
          </div>

          {/* Start Time */}
          <div>
            <label className="block text-xs text-neutral-400 mb-1">Start Time</label>
            <input
              type="time"
              value={startTime}
              onChange={(e) => setStartTime(e.target.value)}
              className="w-full bg-neutral-700 border border-neutral-600 rounded-lg px-3 py-2 text-sm text-neutral-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
            {errors.startTime && <p className="text-xs text-red-400 mt-1">{errors.startTime}</p>}
          </div>

          {/* Duration */}
          <div>
            <label className="block text-xs text-neutral-400 mb-1">Duration (minutes)</label>
            <input
              type="number"
              min={1}
              max={480}
              value={duration}
              onChange={(e) => setDuration(Number(e.target.value))}
              className="w-full bg-neutral-700 border border-neutral-600 rounded-lg px-3 py-2 text-sm text-neutral-100 focus:outline-none focus:ring-1 focus:ring-blue-500"
            />
            {errors.duration && <p className="text-xs text-red-400 mt-1">{errors.duration}</p>}
          </div>

          {/* Notes */}
          <div>
            <label className="block text-xs text-neutral-400 mb-1">Notes</label>
            <textarea
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              rows={2}
              className="w-full bg-neutral-700 border border-neutral-600 rounded-lg px-3 py-2 text-sm text-neutral-100 focus:outline-none focus:ring-1 focus:ring-blue-500 resize-none"
              placeholder="Optional notes"
            />
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm text-neutral-400 hover:text-neutral-200 transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={isSaving}
              className="px-4 py-2 text-sm bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors disabled:opacity-50"
            >
              {isSaving ? 'Saving...' : isEditing ? 'Save' : 'Add Block'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
