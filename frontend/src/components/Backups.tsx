import { useState, useEffect } from 'react';
import { useToast } from '../hooks/useToast';

interface Backup {
  filename: string;
  size_bytes: number;
  created_at: string;
  is_archived: boolean;
}

export default function Backups() {
  const [backups, setBackups] = useState<Backup[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const { showToast } = useToast();

  useEffect(() => {
    fetchBackups();
  }, []);

  const fetchBackups = async () => {
    try {
      setLoading(true);
      const response = await fetch('/api/backups');
      if (!response.ok) throw new Error('Failed to fetch backups');

      const data = await response.json();
      setBackups(data);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      showToast('Failed to load backups', 'error');
    } finally {
      setLoading(false);
    }
  };

  const downloadBackup = async (filename: string) => {
    try {
      const response = await fetch(`/api/backups/${encodeURIComponent(filename)}`);
      if (!response.ok) throw new Error('Failed to download backup');

      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename.split('/').pop() || 'backup.zip';
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);

      showToast('Backup downloaded successfully', 'success');
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Failed to download backup', 'error');
    }
  };

  const deleteBackup = async (filename: string) => {
    if (!confirm(`Are you sure you want to delete backup: ${filename.split('/').pop()}?`)) {
      return;
    }

    try {
      const response = await fetch(`/api/backups/${encodeURIComponent(filename)}`, {
        method: 'DELETE',
      });

      if (!response.ok) throw new Error('Failed to delete backup');

      showToast('Backup deleted successfully', 'success');
      fetchBackups(); // Refresh the list
    } catch (err) {
      showToast(err instanceof Error ? err.message : 'Failed to delete backup', 'error');
    }
  };

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';

    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));

    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="text-center py-8">
          <div className="inline-flex items-center justify-center w-24 h-24 rounded-full bg-adventure-600/20 mb-4">
            <span className="text-5xl">💾</span>
          </div>
          <h2 className="text-3xl font-bold text-white mb-2">Backup Manager</h2>
          <p className="text-adventure-300">Manage your server backups</p>
        </div>

        <div className="bg-white/5 border border-adventure-500/20 rounded-lg p-12 text-center">
          <div className="text-adventure-300">Loading backups...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div className="text-center py-8">
          <div className="inline-flex items-center justify-center w-24 h-24 rounded-full bg-adventure-600/20 mb-4">
            <span className="text-5xl">💾</span>
          </div>
          <h2 className="text-3xl font-bold text-white mb-2">Backup Manager</h2>
          <p className="text-adventure-300">Manage your server backups</p>
        </div>

        <div className="bg-red-500/20 border border-red-500/30 rounded-lg p-6">
          <p className="text-red-200">Error: {error}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="text-center py-8">
        <div className="inline-flex items-center justify-center w-24 h-24 rounded-full bg-adventure-600/20 mb-4">
          <span className="text-5xl">💾</span>
        </div>
        <h2 className="text-3xl font-bold text-white mb-2">Backup Manager</h2>
        <p className="text-adventure-300">Manage your server backups</p>
      </div>

      <div className="bg-white/5 border border-adventure-500/20 rounded-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-xl font-bold text-white">Server Backups ({backups.length})</h3>
          <button
            onClick={fetchBackups}
            className="px-3 py-1 bg-slate-600 hover:bg-slate-700 text-white rounded text-sm font-medium transition-colors"
          >
            🔄 Refresh
          </button>
        </div>

        {backups.length === 0 ? (
          <div className="text-center py-12 text-adventure-300">
            <p className="text-lg mb-2">No backups found</p>
            <p className="text-sm">Backups will appear here when the server creates them</p>
          </div>
        ) : (
          <div className="space-y-2">
            {backups.map((backup) => (
              <div
                key={backup.filename}
                className="bg-black/30 border border-adventure-500/20 rounded-lg p-4 hover:border-adventure-500/40 transition-colors"
              >
                <div className="flex items-center justify-between gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 mb-1">
                      <h4 className="text-white font-medium truncate">
                        {backup.filename.split('/').pop()}
                      </h4>
                      {backup.is_archived && (
                        <span className="px-2 py-0.5 bg-yellow-500/20 text-yellow-200 border border-yellow-500/30 rounded text-xs font-medium">
                          Archived
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-4 text-sm text-adventure-300">
                      <span>📅 {backup.created_at}</span>
                      <span>📦 {formatSize(backup.size_bytes)}</span>
                    </div>
                  </div>

                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => downloadBackup(backup.filename)}
                      className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-medium transition-colors"
                    >
                      ⬇ Download
                    </button>
                    <button
                      onClick={() => deleteBackup(backup.filename)}
                      className="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded font-medium transition-colors"
                    >
                      🗑 Delete
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="bg-white/5 border border-adventure-500/20 rounded-lg p-6">
        <h3 className="text-xl font-bold text-white mb-4">Information</h3>
        <div className="space-y-2 text-adventure-300 text-sm">
          <p>• Backups are created automatically by the server at regular intervals</p>
          <p>• Download backups to save them externally for safekeeping</p>
          <p>• Delete old backups to free up disk space</p>
          <p>• Archived backups are older backups moved to the archive folder</p>
          <p>• <strong className="text-white">Note:</strong> Backup restore functionality coming soon!</p>
        </div>
      </div>
    </div>
  );
}
