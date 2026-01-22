import { useState, useEffect } from 'react';

type LogType = 'hsmm' | 'server' | 'webui';

export default function LogViewer() {
  const [activeLogType, setActiveLogType] = useState<LogType>('server');
  const [logs, setLogs] = useState<string>('');
  const [loading, setLoading] = useState(false);
  const [lineCount, setLineCount] = useState(100);
  const [autoRefresh, setAutoRefresh] = useState(false);

  useEffect(() => {
    loadLogs();
  }, [lineCount, activeLogType]);

  useEffect(() => {
    if (autoRefresh) {
      const interval = setInterval(loadLogs, 2000);
      return () => clearInterval(interval);
    }
  }, [autoRefresh, lineCount, activeLogType]);

  const loadLogs = async () => {
    setLoading(true);
    try {
      const response = await fetch(`/api/logs/${activeLogType}?lines=${lineCount}`);
      if (!response.ok) throw new Error('Failed to load logs');

      const data = await response.json();
      setLogs(data.logs || '');
    } catch (err) {
      setLogs(`Error loading logs: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  const logTabs = [
    { id: 'server' as LogType, label: 'Server Logs', icon: '🎮' },
    { id: 'hsmm' as LogType, label: 'Mod Manager', icon: '📦' },
    { id: 'webui' as LogType, label: 'Web UI', icon: '🌐' },
  ];

  return (
    <div className="space-y-4">
      {/* Log Type Tabs */}
      <div className="flex gap-2 border-b border-adventure-500/20 pb-2">
        {logTabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveLogType(tab.id)}
            className={`
              px-4 py-2 rounded-t font-medium transition-colors
              ${activeLogType === tab.id
                ? 'bg-adventure-600 text-white border-b-2 border-adventure-400'
                : 'text-adventure-300 hover:bg-white/5 hover:text-white'
              }
            `}
          >
            <span className="mr-2">{tab.icon}</span>
            {tab.label}
          </button>
        ))}
      </div>

      {/* Controls */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <label className="text-white font-medium">Lines:</label>
          <select
            value={lineCount}
            onChange={(e) => setLineCount(Number(e.target.value))}
            className="px-3 py-2 bg-white/10 border border-adventure-500/30 rounded text-white focus:outline-none focus:border-adventure-400"
          >
            <option value={50}>50</option>
            <option value={100}>100</option>
            <option value={200}>200</option>
            <option value={500}>500</option>
            <option value={1000}>1000</option>
          </select>

          <button
            onClick={() => setAutoRefresh(!autoRefresh)}
            className={`
              px-4 py-2 rounded font-medium transition-colors
              ${autoRefresh
                ? 'bg-green-600 hover:bg-green-700 text-white'
                : 'bg-slate-600 hover:bg-slate-700 text-white'
              }
            `}
          >
            {autoRefresh ? '⏸️ Stop Auto-Refresh' : '▶️ Auto-Refresh'}
          </button>
        </div>

        <button
          onClick={loadLogs}
          disabled={loading}
          className="px-4 py-2 bg-adventure-600 hover:bg-adventure-700 disabled:bg-gray-600 text-white rounded font-medium transition-colors"
        >
          🔄 Refresh Now
        </button>
      </div>

      {/* Log Display */}
      <div className="relative">
        <div className="bg-black/60 border border-adventure-500/30 rounded-lg p-4 h-[600px] overflow-auto font-mono text-xs">
          <pre className="text-green-300 whitespace-pre">
            {logs || 'No logs available'}
          </pre>
        </div>

        {loading && (
          <div className="absolute top-4 right-4 bg-adventure-600/80 text-white px-3 py-1 rounded text-sm">
            Loading...
          </div>
        )}
      </div>

      {/* Info */}
      <div className="bg-blue-500/20 border border-blue-500/30 rounded-lg p-3 text-sm text-blue-200">
        <strong>💡 Tip:</strong> Enable auto-refresh to see logs update in real-time. Switch between tabs to view different log sources.
      </div>
    </div>
  );
}
