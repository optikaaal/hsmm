import { useState } from 'react';
import { ToastProvider } from './components/ui/Toast';
import ModBrowser from './components/ModBrowser';
import InstalledMods from './components/InstalledMods';
import ConfigEditor from './components/ConfigEditor';
import ServerControl from './components/ServerControl';
import LogViewer from './components/LogViewer';

type Tab = 'browse' | 'installed' | 'config' | 'server' | 'logs';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('browse');

  const tabs = [
    { id: 'browse' as Tab, label: 'Browse Mods', icon: '🔍', description: 'Discover new mods' },
    { id: 'installed' as Tab, label: 'Installed Mods', icon: '📦', description: 'Manage your mods' },
    { id: 'config' as Tab, label: 'Configuration', icon: '⚙️', description: 'Edit server config' },
    { id: 'server' as Tab, label: 'Server Control', icon: '🎮', description: 'Control server' },
    { id: 'logs' as Tab, label: 'Logs', icon: '📋', description: 'View server logs' },
  ];

  return (
    <ToastProvider>
    <div className="min-h-screen bg-gradient-to-br from-stone-900 via-stone-800 to-stone-900 pb-8">
      {/* Sticky Header + Navigation */}
      <div className="sticky top-0 z-50">
        {/* Header */}
        <header className="bg-gradient-to-r from-wood-900/90 to-stone-900/90 backdrop-blur-md border-b border-adventure-600/30 shadow-lg shadow-black/20">
          <div className="max-w-7xl mx-auto px-4 py-4">
            <div className="flex items-center gap-3">
              <div className="text-4xl">⛏️</div>
              <div>
                <h1 className="text-2xl font-bold text-white tracking-tight">Hytale Server Mod Manager</h1>
                <p className="text-adventure-300 text-sm">Manage your Hytale server mods with ease</p>
              </div>
            </div>
          </div>
        </header>

        {/* Navigation Tabs */}
        <nav className="bg-stone-900/80 backdrop-blur-md border-b border-adventure-700/30 shadow-lg shadow-black/10">
          <div className="max-w-7xl mx-auto px-4">
            <div className="flex gap-1">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`
                    relative px-6 py-3 font-medium transition-all duration-300 group
                    ${activeTab === tab.id
                      ? 'text-white'
                      : 'text-stone-300 hover:text-white'
                    }
                  `}
                >
                  {/* Active indicator bar */}
                  {activeTab === tab.id && (
                    <div className="absolute inset-0 bg-gradient-to-br from-adventure-600 to-adventure-700 rounded-t-lg shadow-lg shadow-adventure-500/50 transition-all duration-300" />
                  )}

                  {/* Hover effect */}
                  {activeTab !== tab.id && (
                    <div className="absolute inset-0 bg-adventure-900/20 rounded-t-lg opacity-0 group-hover:opacity-100 transition-opacity duration-200" />
                  )}

                  {/* Content */}
                  <span className="relative flex items-center gap-2">
                    <span className="text-lg">{tab.icon}</span>
                    <span className="hidden sm:inline">{tab.label}</span>
                  </span>

                  {/* Active underline with glow */}
                  {activeTab === tab.id && (
                    <div className="absolute bottom-0 left-0 right-0 h-1 bg-gradient-to-r from-adventure-400 via-adventure-300 to-adventure-400 shadow-lg shadow-adventure-400/50" />
                  )}
                </button>
              ))}
            </div>
          </div>
        </nav>
      </div>

      {/* Main Content with fade transition */}
      <main className="max-w-7xl mx-auto px-4 py-6">
        <div className="bg-stone-900/40 backdrop-blur-sm rounded-lg border border-adventure-700/30 p-6 min-h-[600px] shadow-xl shadow-black/20 animate-in fade-in duration-300">
          {activeTab === 'browse' && <ModBrowser />}
          {activeTab === 'installed' && <InstalledMods />}
          {activeTab === 'config' && <ConfigEditor />}
          {activeTab === 'server' && <ServerControl />}
          {activeTab === 'logs' && <LogViewer />}
        </div>
      </main>
    </div>
    </ToastProvider>
  );
}

export default App;
