import { useState, useEffect } from 'react';

type ConfigFile = 'config.json' | 'permissions.json' | 'bans.json' | 'whitelist.json';

export default function ConfigEditor() {
  const [selectedFile, setSelectedFile] = useState<ConfigFile>('config.json');
  const [content, setContent] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  const configFiles: { id: ConfigFile; label: string; icon: string }[] = [
    { id: 'config.json', label: 'Server Config', icon: '⚙️' },
    { id: 'permissions.json', label: 'Permissions', icon: '🔐' },
    { id: 'bans.json', label: 'Bans', icon: '🚫' },
    { id: 'whitelist.json', label: 'Whitelist', icon: '✅' },
  ];

  useEffect(() => {
    loadConfig();
  }, [selectedFile]);

  const loadConfig = async () => {
    setLoading(true);
    setError(null);
    setSaved(false);
    try {
      const response = await fetch(`/api/config/${selectedFile}`);
      if (!response.ok) {
        if (response.status === 404) {
          setContent('{}');
        } else {
          throw new Error('Failed to load config file');
        }
      } else {
        const data = await response.text();
        // Pretty print JSON
        try {
          const parsed = JSON.parse(data);
          setContent(JSON.stringify(parsed, null, 2));
        } catch {
          setContent(data);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const saveConfig = async () => {
    // Validate JSON
    try {
      JSON.parse(content);
    } catch {
      alert('Invalid JSON! Please fix syntax errors before saving.');
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const response = await fetch(`/api/config/${selectedFile}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: content,
      });

      if (!response.ok) throw new Error('Failed to save config file');

      setSaved(true);
      setTimeout(() => setSaved(false), 3000);
    } catch (err) {
      alert(`Failed to save: ${err instanceof Error ? err.message : 'Unknown error'}`);
    } finally {
      setLoading(false);
    }
  };

  const formatJson = () => {
    try {
      const parsed = JSON.parse(content);
      setContent(JSON.stringify(parsed, null, 2));
    } catch {
      alert('Invalid JSON! Cannot format.');
    }
  };

  return (
    <div className="space-y-6">
      {/* File Selector */}
      <div className="flex gap-2">
        {configFiles.map((file) => (
          <button
            key={file.id}
            onClick={() => setSelectedFile(file.id)}
            className={`
              px-4 py-2 rounded-lg font-medium transition-colors
              ${selectedFile === file.id
                ? 'bg-adventure-600 text-white'
                : 'bg-white/5 text-adventure-300 hover:bg-white/10'
              }
            `}
          >
            <span className="mr-2">{file.icon}</span>
            {file.label}
          </button>
        ))}
      </div>

      {/* Editor Area */}
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-bold text-white">Editing: {selectedFile}</h2>
          <div className="flex gap-2">
            <button
              onClick={formatJson}
              disabled={loading}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 text-white rounded font-medium transition-colors"
            >
              Format JSON
            </button>
            <button
              onClick={loadConfig}
              disabled={loading}
              className="px-4 py-2 bg-slate-600 hover:bg-slate-700 disabled:bg-gray-600 text-white rounded font-medium transition-colors"
            >
              🔄 Reload
            </button>
            <button
              onClick={saveConfig}
              disabled={loading}
              className="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:bg-gray-600 text-white rounded font-medium transition-colors"
            >
              {loading ? 'Saving...' : '💾 Save'}
            </button>
          </div>
        </div>

        {saved && (
          <div className="bg-green-500/20 border border-green-500/50 rounded-lg p-3 text-green-200">
            Saved successfully!
          </div>
        )}

        {error && (
          <div className="bg-red-500/20 border border-red-500/50 rounded-lg p-3 text-red-200">
            Error: {error}
          </div>
        )}

        {/* JSON Editor */}
        <div className="relative">
          <textarea
            value={content}
            onChange={(e) => setContent(e.target.value)}
            disabled={loading}
            className="w-full h-[500px] px-4 py-3 bg-black/40 border border-adventure-500/30 rounded-lg text-white font-mono text-sm focus:outline-none focus:border-adventure-400 resize-none"
            placeholder="Loading..."
          />
          {loading && (
            <div className="absolute inset-0 bg-black/50 flex items-center justify-center rounded-lg">
              <div className="inline-block w-12 h-12 border-4 border-adventure-500/30 border-t-adventure-500 rounded-full animate-spin"></div>
            </div>
          )}
        </div>

        <div className="bg-blue-500/20 border border-blue-500/30 rounded-lg p-3 text-sm text-blue-200">
          <strong>💡 Tip:</strong> Make sure your JSON is valid before saving. Use the "Format JSON" button to auto-format your changes.
        </div>
      </div>
    </div>
  );
}
