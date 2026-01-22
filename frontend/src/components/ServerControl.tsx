import { useState, useEffect } from 'react';

interface ServerStatus {
  running: boolean;
  status: string;
}

export default function ServerControl() {
  const [status, setStatus] = useState<ServerStatus | null>(null);
  const [restarting, setRestarting] = useState(false);
  const [stopping, setStopping] = useState(false);
  const [starting, setStarting] = useState(false);

  useEffect(() => {
    checkStatus();
    // Poll status every 5 seconds
    const interval = setInterval(checkStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const checkStatus = async () => {
    try {
      const response = await fetch('/api/server/status');
      if (!response.ok) throw new Error('Failed to check server status');

      const data = await response.json();
      setStatus(data);
    } catch (err) {
      console.error('Failed to check status:', err);
    }
  };

  const restartServer = async () => {
    if (!confirm('Are you sure you want to restart the Hytale server?')) return;

    setRestarting(true);
    try {
      const response = await fetch('/api/server/restart', {
        method: 'POST',
      });

      if (!response.ok) throw new Error('Failed to restart server');

      const data = await response.json();
      alert(data.message || 'Server restart initiated');

      // Check status after a delay
      setTimeout(checkStatus, 3000);
    } catch (err) {
      alert(`Failed to restart: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setRestarting(false);
    }
  };

  const stopServer = async () => {
    if (!confirm('Are you sure you want to stop the Hytale server? You can start it again after making changes.')) return;

    setStopping(true);
    try {
      const response = await fetch('/api/server/stop', {
        method: 'POST',
      });

      if (!response.ok) throw new Error('Failed to stop server');

      const data = await response.json();
      alert(`${data.message}\n${data.note || ''}`);

      // Check status after a delay
      setTimeout(checkStatus, 2000);
    } catch (err) {
      alert(`Failed to stop: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setStopping(false);
    }
  };

  const startServer = async () => {
    if (!confirm('Start the Hytale server?')) return;

    setStarting(true);
    try {
      const response = await fetch('/api/server/start', {
        method: 'POST',
      });

      if (!response.ok) throw new Error('Failed to start server');

      const data = await response.json();
      alert(`${data.message}\n${data.note || ''}`);

      // Check status after a delay
      setTimeout(checkStatus, 3000);
    } catch (err) {
      alert(`Failed to start: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setStarting(false);
    }
  };

  return (
    <div className="space-y-6">
      <div className="text-center py-8">
        <div className="inline-flex items-center justify-center w-24 h-24 rounded-full bg-adventure-600/20 mb-4">
          <span className="text-5xl">🎮</span>
        </div>
        <h2 className="text-3xl font-bold text-white mb-2">Server Control Panel</h2>
        <p className="text-adventure-300">Manage your Hytale server</p>
      </div>

      {/* Server Status */}
      <div className="bg-white/5 border border-adventure-500/20 rounded-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-xl font-bold text-white">Server Status</h3>
          <button
            onClick={checkStatus}
            className="px-3 py-1 bg-slate-600 hover:bg-slate-700 text-white rounded text-sm font-medium transition-colors"
          >
            🔄 Refresh
          </button>
        </div>

        {status && (
          <div className="space-y-3">
            <div className="flex items-center gap-3">
              <div className={`
                w-4 h-4 rounded-full animate-pulse
                ${status.running ? 'bg-green-500' : 'bg-red-500'}
              `}></div>
              <span className="text-lg text-white font-medium">
                {status.running ? 'Server is Running' : 'Server is Offline'}
              </span>
            </div>

            <div className="bg-black/30 rounded p-3 font-mono text-sm">
              <div className="text-adventure-300">Status: <span className="text-white">{status.status}</span></div>
            </div>
          </div>
        )}

        {!status && (
          <div className="text-center py-4 text-adventure-300">
            Loading status...
          </div>
        )}
      </div>

      {/* Server Actions */}
      <div className="bg-white/5 border border-adventure-500/20 rounded-lg p-6">
        <h3 className="text-xl font-bold text-white mb-4">Server Actions</h3>

        <div className="space-y-3">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <button
              onClick={stopServer}
              disabled={stopping || !status?.running}
              className="px-6 py-4 bg-red-600 hover:bg-red-700 disabled:bg-gray-600 text-white rounded-lg font-bold text-lg transition-colors flex items-center justify-center gap-2"
            >
              <span>⏹</span>
              <span>{stopping ? 'Stopping...' : 'Stop Server'}</span>
            </button>

            <button
              onClick={startServer}
              disabled={starting || status?.running}
              className="px-6 py-4 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 text-white rounded-lg font-bold text-lg transition-colors flex items-center justify-center gap-2"
            >
              <span>▶</span>
              <span>{starting ? 'Starting...' : 'Start Server'}</span>
            </button>
          </div>

          <button
            onClick={restartServer}
            disabled={restarting}
            className="w-full px-6 py-4 bg-orange-600 hover:bg-orange-700 disabled:bg-gray-600 text-white rounded-lg font-bold text-lg transition-colors flex items-center justify-center gap-2"
          >
            <span>🔄</span>
            <span>{restarting ? 'Restarting Server...' : 'Restart Server'}</span>
          </button>

          <div className="bg-yellow-500/20 border border-yellow-500/30 rounded-lg p-4 text-sm text-yellow-200">
            <strong>⚠️ Warning:</strong> Stopping or restarting the server will disconnect all players. Make sure to save your progress first!
          </div>

          <div className="bg-blue-500/20 border border-blue-500/30 rounded-lg p-4 text-sm text-blue-200">
            <strong>💡 Tip:</strong> Stop the server to safely edit config files and download/update mods, then start it back up to apply changes without container restart.
          </div>
        </div>
      </div>

      {/* Information */}
      <div className="bg-white/5 border border-adventure-500/20 rounded-lg p-6">
        <h3 className="text-xl font-bold text-white mb-4">Information</h3>
        <div className="space-y-2 text-adventure-300 text-sm">
          <p>• <strong className="text-white">Stop Server:</strong> Safely stop the server to edit configs or manage mods</p>
          <p>• <strong className="text-white">Start Server:</strong> Start the stopped server with updated configurations</p>
          <p>• <strong className="text-white">Restart Server:</strong> Quick restart that updates mods automatically</p>
          <p>• Config files are read on startup, so stop the server before manual edits</p>
          <p>• The server status updates automatically every 5 seconds</p>
          <p>• Make sure to enable/disable mods before starting for changes to apply</p>
        </div>
      </div>
    </div>
  );
}
