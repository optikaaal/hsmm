import { useState, useEffect } from 'react';
import { useToast } from './ui/Toast';
import ConfirmDialog from './ui/ConfirmDialog';
import StatusPill from './ui/StatusPill';
import Badge from './ui/Badge';

interface InstalledMod {
  name: string;
  enabled: boolean;
  project_id: number;
  installed: boolean;
  version: string | null;
}

export default function InstalledMods() {
  const { showToast } = useToast();
  const [mods, setMods] = useState<InstalledMod[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedMods, setSelectedMods] = useState<Set<string>>(new Set());
  const [confirmDialog, setConfirmDialog] = useState<{
    isOpen: boolean;
    title: string;
    message: string;
    action: () => void;
    variant: 'danger' | 'warning' | 'info';
  }>({
    isOpen: false,
    title: '',
    message: '',
    action: () => {},
    variant: 'info',
  });

  useEffect(() => {
    loadMods();
  }, []);

  const loadMods = async () => {
    setLoading(true);
    setError(null);
    try {
      const response = await fetch('/api/mods');
      if (!response.ok) throw new Error('Failed to load installed mods');

      const data = await response.json();
      setMods(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const toggleMod = async (name: string, currentlyEnabled: boolean) => {
    try {
      const response = await fetch(`/api/mods/${encodeURIComponent(name)}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: !currentlyEnabled }),
      });

      if (!response.ok) throw new Error('Failed to toggle mod');

      showToast(
        `${name} has been ${!currentlyEnabled ? 'enabled' : 'disabled'}`,
        !currentlyEnabled ? 'success' : 'info'
      );
      await loadMods();
    } catch (err) {
      showToast(`Failed to toggle mod: ${err instanceof Error ? err.message : 'Unknown error'}`, 'error');
    }
  };

  const removeMod = async (name: string) => {
    try {
      const response = await fetch(`/api/mods/${encodeURIComponent(name)}`, {
        method: 'DELETE',
      });

      if (!response.ok) throw new Error('Failed to remove mod');

      showToast(`${name} has been removed`, 'success');
      await loadMods();
    } catch (err) {
      showToast(`Failed to remove mod: ${err instanceof Error ? err.message : 'Unknown error'}`, 'error');
    }
  };

  // Bulk selection
  const toggleSelectAll = () => {
    if (selectedMods.size === mods.length) {
      setSelectedMods(new Set());
    } else {
      setSelectedMods(new Set(mods.map(m => m.name)));
    }
  };

  const toggleSelectMod = (name: string) => {
    const newSelected = new Set(selectedMods);
    if (newSelected.has(name)) {
      newSelected.delete(name);
    } else {
      newSelected.add(name);
    }
    setSelectedMods(newSelected);
  };

  // Bulk actions
  const bulkEnable = async () => {
    const promises = Array.from(selectedMods).map(name => {
      const mod = mods.find(m => m.name === name);
      if (mod && !mod.enabled) {
        return fetch(`/api/mods/${encodeURIComponent(name)}`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ enabled: true }),
        });
      }
      return null;
    }).filter(p => p !== null);

    try {
      await Promise.all(promises);
      showToast(`${promises.length} mod(s) enabled`, 'success');
      await loadMods();
      setSelectedMods(new Set());
    } catch (err) {
      showToast('Failed to enable some mods', 'error');
    }
  };

  const bulkDisable = async () => {
    const promises = Array.from(selectedMods).map(name => {
      const mod = mods.find(m => m.name === name);
      if (mod && mod.enabled) {
        return fetch(`/api/mods/${encodeURIComponent(name)}`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ enabled: false }),
        });
      }
      return null;
    }).filter(p => p !== null);

    try {
      await Promise.all(promises);
      showToast(`${promises.length} mod(s) disabled`, 'info');
      await loadMods();
      setSelectedMods(new Set());
    } catch (err) {
      showToast('Failed to disable some mods', 'error');
    }
  };

  const bulkRemove = async () => {
    const promises = Array.from(selectedMods).map(name => {
      return fetch(`/api/mods/${encodeURIComponent(name)}`, {
        method: 'DELETE',
      });
    });

    try {
      await Promise.all(promises);
      showToast(`${promises.length} mod(s) removed`, 'success');
      await loadMods();
      setSelectedMods(new Set());
    } catch (err) {
      showToast('Failed to remove some mods', 'error');
    }
  };

  // Confirm dialog helpers
  const confirmBulkRemove = () => {
    setConfirmDialog({
      isOpen: true,
      title: 'Remove Selected Mods',
      message: `Are you sure you want to remove ${selectedMods.size} selected mod(s)? This action cannot be undone.`,
      action: bulkRemove,
      variant: 'danger',
    });
  };

  const confirmSingleRemove = (name: string) => {
    setConfirmDialog({
      isOpen: true,
      title: 'Remove Mod',
      message: `Are you sure you want to remove "${name}"? This action cannot be undone.`,
      action: () => removeMod(name),
      variant: 'danger',
    });
  };

  if (loading) {
    return (
      <div className="text-center py-16">
        <div className="inline-block w-12 h-12 border-4 border-adventure-500/30 border-t-adventure-500 rounded-full animate-spin"></div>
        <p className="mt-4 text-adventure-300 font-medium">Loading installed mods...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-500/20 border border-red-500/50 rounded-lg p-4 text-red-200">
        <strong className="font-semibold">Error:</strong> {error}
      </div>
    );
  }

  if (mods.length === 0) {
    return (
      <div className="text-center py-16">
        <div className="text-7xl mb-4">📦</div>
        <p className="text-2xl text-adventure-300 font-semibold mb-2">No mods installed yet</p>
        <p className="text-sm text-adventure-400">Visit the Browse Mods tab to discover and install mods</p>
      </div>
    );
  }

  const enabledCount = mods.filter(m => m.enabled).length;
  const disabledCount = mods.filter(m => !m.enabled).length;

  return (
    <div className="space-y-6">
      {/* Header with Stats */}
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <h2 className="text-3xl font-bold text-white mb-2">Installed Mods</h2>
          <div className="flex items-center gap-3">
            <Badge variant="success" size="lg">
              ✓ {enabledCount} Enabled
            </Badge>
            <Badge variant="default" size="lg">
              ⏸ {disabledCount} Disabled
            </Badge>
            <Badge variant="info" size="lg">
              📦 {mods.length} Total
            </Badge>
          </div>
        </div>
        <button
          onClick={loadMods}
          className="px-4 py-2.5 bg-adventure-600 hover:bg-adventure-700 text-white rounded-lg font-medium transition-colors shadow-lg shadow-adventure-500/30"
        >
          🔄 Refresh
        </button>
      </div>

      {/* Bulk Actions Bar */}
      {selectedMods.size > 0 && (
        <div className="bg-adventure-600/20 border border-adventure-500/40 rounded-lg p-4 animate-in slide-in-from-top duration-300">
          <div className="flex flex-wrap items-center gap-3">
            <span className="text-white font-semibold">
              {selectedMods.size} mod(s) selected
            </span>
            <div className="h-6 w-px bg-adventure-500/40"></div>
            <button
              onClick={bulkEnable}
              className="px-4 py-2 bg-green-600 hover:bg-green-700 text-white rounded-lg text-sm font-medium transition-colors"
            >
              ✓ Enable All
            </button>
            <button
              onClick={bulkDisable}
              className="px-4 py-2 bg-stone-600 hover:bg-stone-700 text-white rounded-lg text-sm font-medium transition-colors"
            >
              ⏸ Disable All
            </button>
            <button
              onClick={confirmBulkRemove}
              className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm font-medium transition-colors"
            >
              🗑 Remove All
            </button>
            <button
              onClick={() => setSelectedMods(new Set())}
              className="ml-auto px-4 py-2 bg-white/10 hover:bg-white/20 text-white rounded-lg text-sm font-medium transition-colors"
            >
              Clear Selection
            </button>
          </div>
        </div>
      )}

      {/* Mods List */}
      <div className="space-y-3">
        {/* Select All Header */}
        <div className="bg-white/5 border border-adventure-500/20 rounded-lg p-4">
          <label className="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              checked={selectedMods.size === mods.length}
              onChange={toggleSelectAll}
              className="w-5 h-5 rounded border-adventure-500/30 bg-white/10 text-adventure-600 focus:ring-adventure-500 cursor-pointer"
            />
            <span className="text-white font-semibold">
              {selectedMods.size === mods.length ? 'Deselect All' : 'Select All'}
            </span>
          </label>
        </div>

        {/* Mod Cards */}
        {mods.map((mod) => (
          <div
            key={mod.name}
            className={`
              bg-white/5 border rounded-xl p-4 transition-all duration-200
              ${selectedMods.has(mod.name)
                ? 'border-adventure-500 bg-adventure-500/10 shadow-lg shadow-adventure-500/20'
                : mod.enabled
                  ? 'border-green-500/40 bg-green-500/5'
                  : 'border-gray-500/40'
              }
              hover:bg-white/10
            `}
          >
            <div className="flex items-center gap-4">
              {/* Checkbox */}
              <input
                type="checkbox"
                checked={selectedMods.has(mod.name)}
                onChange={() => toggleSelectMod(mod.name)}
                className="w-5 h-5 rounded border-adventure-500/30 bg-white/10 text-adventure-600 focus:ring-adventure-500 cursor-pointer flex-shrink-0"
              />

              {/* Status Indicator */}
              <div
                className={`
                  w-3 h-3 rounded-full flex-shrink-0
                  ${mod.enabled ? 'bg-green-500 shadow-lg shadow-green-500/50' : 'bg-gray-500'}
                `}
              ></div>

              {/* Mod Info */}
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1 flex-wrap">
                  <h3 className="text-lg font-bold text-white">{mod.name}</h3>
                  <StatusPill status={mod.enabled ? 'enabled' : 'disabled'} />
                  {mod.installed && (
                    <Badge variant="info" size="sm">
                      Installed
                    </Badge>
                  )}
                </div>
                <div className="flex items-center gap-4 text-sm text-adventure-300">
                  <span>ID: {mod.project_id}</span>
                  {mod.version && <span>Version: {mod.version}</span>}
                </div>
              </div>

              {/* Actions */}
              <div className="flex items-center gap-2 flex-shrink-0">
                {/* Toggle Switch */}
                <button
                  onClick={() => toggleMod(mod.name, mod.enabled)}
                  className={`
                    relative inline-flex h-7 w-12 items-center rounded-full transition-colors
                    ${mod.enabled ? 'bg-green-600 shadow-lg shadow-green-600/30' : 'bg-gray-600'}
                  `}
                  aria-label={mod.enabled ? 'Disable mod' : 'Enable mod'}
                >
                  <span
                    className={`
                      inline-block h-5 w-5 transform rounded-full bg-white transition-transform shadow-md
                      ${mod.enabled ? 'translate-x-6' : 'translate-x-1'}
                    `}
                  />
                </button>

                {/* Remove Button */}
                <button
                  onClick={() => confirmSingleRemove(mod.name)}
                  className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-lg text-sm font-medium transition-colors shadow-lg shadow-red-600/30"
                >
                  Remove
                </button>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Info Banner */}
      <div className="bg-diamond-500/20 border border-diamond-500/40 rounded-lg p-4">
        <div className="flex items-start gap-3">
          <span className="text-2xl">ℹ️</span>
          <div className="flex-1">
            <p className="text-diamond-200 font-medium mb-1">Server Restart Required</p>
            <p className="text-sm text-diamond-300/80">
              Changes to mods may require a server restart to take effect. Visit the Server Control tab to restart your server.
            </p>
          </div>
        </div>
      </div>

      {/* Confirm Dialog */}
      <ConfirmDialog
        isOpen={confirmDialog.isOpen}
        onClose={() => setConfirmDialog({ ...confirmDialog, isOpen: false })}
        onConfirm={confirmDialog.action}
        title={confirmDialog.title}
        message={confirmDialog.message}
        variant={confirmDialog.variant}
        confirmText="Confirm"
        cancelText="Cancel"
      />
    </div>
  );
}
