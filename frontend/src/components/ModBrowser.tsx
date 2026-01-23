import { useState, useEffect } from 'react';
import { useToast } from '../hooks/useToast';
import Modal from './ui/Modal';
import Badge from './ui/Badge';
import StatusPill from './ui/StatusPill';

interface Mod {
  id: number;
  name: string;
  summary: string;
  download_count: number;
  logo_url: string | null;
  author: string;
  date_modified: string;
  categories?: string[];
}

interface ModDetails {
  id: number;
  name: string;
  summary: string;
  description: string;
  download_count: number;
  logo_url: string | null;
  author: string;
  authors: string[];
  date_created: string;
  date_modified: string;
  date_released: string;
  categories: string[];
  screenshots: Screenshot[];
  latest_files: FileInfo[];
  links: ModLinks;
}

interface Screenshot {
  title: string;
  description: string;
  thumbnail_url: string;
  url: string;
}

interface FileInfo {
  id: number;
  display_name: string;
  file_name: string;
  file_date: string;
  file_length: number;
  download_count: number;
}

interface ModLinks {
  website_url: string | null;
  wiki_url: string | null;
  issues_url: string | null;
  source_url: string | null;
}

interface InstalledMod {
  name: string;
  enabled: boolean;
  project_id: number;
  installed: boolean;
}

type SortOption = 'popular' | 'newest' | 'alphabetical';

export default function ModBrowser() {
  const { showToast } = useToast();
  const [mods, setMods] = useState<Mod[]>([]);
  const [installedMods, setInstalledMods] = useState<InstalledMod[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchInput, setSearchInput] = useState('');
  const [page, setPage] = useState(0);
  const [installing, setInstalling] = useState<number | null>(null);
  const [hideInstalled, setHideInstalled] = useState(false);
  const [sortBy, setSortBy] = useState<SortOption>('popular');
  const [selectedModId, setSelectedModId] = useState<number | null>(null);
  const [modDetails, setModDetails] = useState<ModDetails | null>(null);
  const [loadingDetails, setLoadingDetails] = useState(false);

  // Debounced search effect
  useEffect(() => {
    const timer = setTimeout(() => {
      if (searchInput !== searchQuery) {
        setSearchQuery(searchInput);
        setPage(0);
      }
    }, 500);

    return () => clearTimeout(timer);
  }, [searchInput]);

  useEffect(() => {
    loadMods();
  }, [page, searchQuery, hideInstalled]);

  useEffect(() => {
    loadInstalledMods();
  }, []);

  const loadInstalledMods = async () => {
    try {
      const response = await fetch('/api/mods');
      if (response.ok) {
        const data = await response.json();
        setInstalledMods(data);
      }
    } catch (err) {
      console.error('Failed to load installed mods:', err);
    }
  };

  const loadMods = async () => {
    setLoading(true);
    setError(null);
    try {
      // When hiding installed mods, fetch extra pages to compensate for filtering
      const pagesToFetch = hideInstalled ? 3 : 1;
      const allMods: Mod[] = [];

      for (let i = 0; i < pagesToFetch; i++) {
        const currentPage = page * pagesToFetch + i;
        const url = searchQuery
          ? `/api/curseforge/search?q=${encodeURIComponent(searchQuery)}&page=${currentPage}`
          : `/api/curseforge/popular?page=${currentPage}`;

        const response = await fetch(url);
        if (!response.ok) throw new Error('Failed to load mods');

        const data = await response.json();
        allMods.push(...data);
      }

      setMods(allMods);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const isModInstalled = (modId: number) => {
    return installedMods.some(m => m.project_id === modId);
  };

  const getInstalledMod = (modId: number) => {
    return installedMods.find(m => m.project_id === modId);
  };

  const installMod = async (mod: Mod | ModDetails) => {
    setInstalling(mod.id);
    try {
      const response = await fetch('/api/mods', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: mod.name,
          project_id: mod.id,
          enabled: true,
        }),
      });

      if (!response.ok) throw new Error('Failed to install mod');

      showToast(`${mod.name} installed successfully!`, 'success');
      await loadInstalledMods();
    } catch (err) {
      showToast(`Failed to install: ${err instanceof Error ? err.message : 'Unknown error'}`, 'error');
    } finally {
      setInstalling(null);
    }
  };

  const disableMod = async (mod: Mod, installedMod: InstalledMod) => {
    try {
      const response = await fetch(`/api/mods/${encodeURIComponent(installedMod.name)}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ enabled: false }),
      });

      if (!response.ok) throw new Error('Failed to disable mod');

      showToast(`${mod.name} has been disabled`, 'info');
      await loadInstalledMods();
    } catch (err) {
      showToast(`Failed to disable: ${err instanceof Error ? err.message : 'Unknown error'}`, 'error');
    }
  };

  const loadModDetails = async (modId: number) => {
    setLoadingDetails(true);
    setModDetails(null);
    try {
      const response = await fetch(`/api/curseforge/mod/${modId}`);
      if (!response.ok) throw new Error('Failed to load mod details');

      const data = await response.json();
      setModDetails(data);
    } catch (err) {
      showToast(`Failed to load mod details: ${err instanceof Error ? err.message : 'Unknown error'}`, 'error');
      setSelectedModId(null);
    } finally {
      setLoadingDetails(false);
    }
  };

  const openModDetails = (modId: number) => {
    setSelectedModId(modId);
    loadModDetails(modId);
  };

  const closeModDetails = () => {
    setSelectedModId(null);
    setModDetails(null);
  };

  // Sort and filter mods
  const getSortedMods = () => {
    const sorted = [...mods];

    switch (sortBy) {
      case 'newest':
        sorted.sort((a, b) => new Date(b.date_modified).getTime() - new Date(a.date_modified).getTime());
        break;
      case 'alphabetical':
        sorted.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'popular':
      default:
        // Already sorted by popularity from API
        break;
    }

    return sorted;
  };

  const filteredMods = hideInstalled
    ? getSortedMods().filter(mod => !isModInstalled(mod.id))
    : getSortedMods();

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    // Immediately apply search without waiting for debounce
    if (searchInput !== searchQuery) {
      setSearchQuery(searchInput);
      setPage(0);
    }
  };

  // Helper to format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
  };

  // Helper to strip HTML tags from description
  const stripHtml = (html: string): string => {
    const tmp = document.createElement('DIV');
    tmp.innerHTML = html;
    return tmp.textContent || tmp.innerText || '';
  };

  return (
    <div className="space-y-6">
      {/* Sticky Search & Filter Bar */}
      <div className="sticky top-[140px] z-40 -mx-6 -mt-6 px-6 pt-6 pb-4 bg-black/40 backdrop-blur-md border-b border-adventure-500/20">
        {/* Search Bar */}
        <form onSubmit={handleSearch} className="flex gap-3 mb-4">
          <div className="flex-1 relative">
            <input
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="Search mods... (live search)"
              className="w-full px-4 py-2.5 bg-white/10 border border-adventure-500/30 rounded-lg text-white placeholder-adventure-300/50 focus:outline-none focus:border-adventure-400 focus:ring-2 focus:ring-adventure-400/20 transition-all"
            />
            {searchInput && searchInput !== searchQuery && (
              <div className="absolute right-3 top-1/2 -translate-y-1/2">
                <div className="w-4 h-4 border-2 border-adventure-500/30 border-t-adventure-500 rounded-full animate-spin"></div>
              </div>
            )}
          </div>
          <button
            type="submit"
            className="px-6 py-2.5 bg-adventure-600 hover:bg-adventure-700 text-white rounded-lg font-medium transition-colors shadow-lg shadow-adventure-500/30"
          >
            🔍 Search
          </button>
          {(searchQuery || searchInput) && (
            <button
              type="button"
              onClick={() => {
                setSearchInput('');
                setSearchQuery('');
                setPage(0);
              }}
              className="px-4 py-2.5 bg-stone-600 hover:bg-stone-700 text-white rounded-lg transition-colors"
            >
              Clear
            </button>
          )}
        </form>

        {/* Filters & Sort */}
        <div className="flex flex-wrap items-center gap-4">
          {/* Hide Installed Filter */}
          <label className="flex items-center gap-2 text-white cursor-pointer hover:text-adventure-300 transition-colors">
            <input
              type="checkbox"
              checked={hideInstalled}
              onChange={(e) => {
                setHideInstalled(e.target.checked);
                setPage(0);
              }}
              className="w-4 h-4 rounded border-adventure-500/30 bg-white/10 text-adventure-600 focus:ring-adventure-500 cursor-pointer"
            />
            <span className="text-sm font-medium">Hide installed mods</span>
          </label>

          {/* Divider */}
          <div className="h-6 w-px bg-adventure-500/30"></div>

          {/* Sort Options */}
          <div className="flex items-center gap-2">
            <span className="text-sm text-adventure-300 font-medium">Sort by:</span>
            <div className="flex gap-2">
              {[
                { value: 'popular', label: 'Popular', icon: '🔥' },
                { value: 'newest', label: 'Newest', icon: '⭐' },
                { value: 'alphabetical', label: 'A-Z', icon: '🔤' },
              ].map((option) => (
                <button
                  key={option.value}
                  onClick={() => setSortBy(option.value as SortOption)}
                  className={`
                    px-3 py-1.5 rounded-lg text-sm font-medium transition-all
                    ${sortBy === option.value
                      ? 'bg-adventure-600 text-white shadow-lg shadow-adventure-500/30'
                      : 'bg-white/5 text-adventure-300 hover:bg-white/10 hover:text-white'
                    }
                  `}
                >
                  <span className="mr-1.5">{option.icon}</span>
                  {option.label}
                </button>
              ))}
            </div>
          </div>

          {/* Results Count */}
          <div className="ml-auto text-sm text-adventure-300">
            <span className="font-semibold text-white">{filteredMods.length}</span> mods found
          </div>
        </div>
      </div>

      {/* Loading State */}
      {loading && (
        <div className="text-center py-16">
          <div className="inline-block w-12 h-12 border-4 border-adventure-500/30 border-t-adventure-500 rounded-full animate-spin"></div>
          <p className="mt-4 text-adventure-300 font-medium">Loading mods...</p>
        </div>
      )}

      {/* Error State */}
      {error && (
        <div className="bg-red-500/20 border border-red-500/50 rounded-lg p-4 text-red-200">
          <strong className="font-semibold">Error:</strong> {error}
        </div>
      )}

      {/* Mods Grid */}
      {!loading && !error && (
        <>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
            {filteredMods.map((mod) => {
              const installed = isModInstalled(mod.id);
              const installedMod = getInstalledMod(mod.id);

              return (
                <div
                  key={mod.id}
                  className={`
                    bg-white/5 border rounded-xl p-5 transition-all duration-300 flex flex-col h-full
                    hover:bg-white/10 hover:border-adventure-400/40 hover:shadow-lg hover:shadow-adventure-500/20 hover:-translate-y-0.5
                    ${installed ? 'border-green-500/30' : 'border-adventure-500/20'}
                  `}
                >
                  {/* Mod Header */}
                  <div className="flex items-start gap-3 mb-4">
                    {mod.logo_url ? (
                      <img
                        src={mod.logo_url}
                        alt={mod.name}
                        className="w-16 h-16 rounded-lg object-cover flex-shrink-0 shadow-md"
                      />
                    ) : (
                      <div className="w-16 h-16 flex-shrink-0 rounded-lg bg-gradient-to-br from-adventure-600 to-adventure-700 flex items-center justify-center text-3xl shadow-md">
                        📦
                      </div>
                    )}
                    <div className="flex-1 min-w-0">
                      <h3 className="text-xl font-bold text-white mb-1 leading-tight">
                        {mod.name}
                      </h3>
                      <p className="text-sm text-adventure-300">by {mod.author}</p>
                    </div>
                  </div>

                  {/* Mod Summary */}
                  <p className="text-sm text-gray-300 mb-4 line-clamp-3 leading-relaxed flex-grow">
                    {mod.summary}
                  </p>

                  {/* Mod Metadata */}
                  <div className="space-y-3 mb-4">
                    <div className="flex items-center justify-between text-sm">
                      <div className="flex items-center gap-1.5 text-adventure-300">
                        <span>⬇️</span>
                        <span className="font-medium">{mod.download_count.toLocaleString()}</span>
                      </div>
                      <div className="flex items-center gap-1.5 text-adventure-300">
                        <span>📅</span>
                        <span className="font-medium">{new Date(mod.date_modified).toLocaleDateString()}</span>
                      </div>
                    </div>

                    {/* Status Badge */}
                    {installed && installedMod && (
                      <div>
                        <StatusPill
                          status={installedMod.enabled ? 'enabled' : 'disabled'}
                          label={installedMod.enabled ? 'Installed & Enabled' : 'Installed (Disabled)'}
                        />
                      </div>
                    )}
                  </div>

                  {/* Action Buttons */}
                  <div className="flex gap-2">
                    {installed && installedMod ? (
                      <>
                        <button
                          onClick={() => openModDetails(mod.id)}
                          className="flex-1 px-4 py-2.5 bg-adventure-600/20 border border-adventure-500/40 hover:bg-adventure-600/30 text-stone-300 hover:text-white rounded-lg font-medium transition-colors"
                        >
                          📄 Details
                        </button>
                        {installedMod.enabled && (
                          <button
                            onClick={() => disableMod(mod, installedMod)}
                            className="px-4 py-2.5 bg-stone-600 hover:bg-stone-700 text-white rounded-lg font-medium transition-colors"
                          >
                            Disable
                          </button>
                        )}
                      </>
                    ) : (
                      <>
                        <button
                          onClick={() => openModDetails(mod.id)}
                          className="px-4 py-2.5 bg-adventure-600/20 border border-adventure-500/40 hover:bg-adventure-600/30 text-stone-300 hover:text-white rounded-lg font-medium transition-colors"
                        >
                          📄
                        </button>
                        <button
                          onClick={() => installMod(mod)}
                          disabled={installing === mod.id}
                          className="flex-1 px-4 py-2.5 bg-adventure-600 hover:bg-adventure-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors shadow-lg shadow-adventure-500/30"
                        >
                          {installing === mod.id ? (
                            <span className="flex items-center justify-center gap-2">
                              <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                              Installing...
                            </span>
                          ) : (
                            '+ Install Mod'
                          )}
                        </button>
                      </>
                    )}
                  </div>
                </div>
              );
            })}
          </div>

          {/* Pagination */}
          <div className="flex justify-center items-center gap-4 mt-8">
            <button
              onClick={() => setPage(Math.max(0, page - 1))}
              disabled={page === 0}
              className="px-5 py-2.5 bg-adventure-600 hover:bg-adventure-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors shadow-lg shadow-adventure-500/30 disabled:shadow-none"
            >
              ← Previous
            </button>
            <span className="px-5 py-2.5 bg-white/5 text-white rounded-lg font-semibold border border-adventure-500/20">
              Page {page + 1}
            </span>
            <button
              onClick={() => setPage(page + 1)}
              disabled={mods.length < 20}
              className="px-5 py-2.5 bg-adventure-600 hover:bg-adventure-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors shadow-lg shadow-adventure-500/30 disabled:shadow-none"
            >
              Next →
            </button>
          </div>
        </>
      )}

      {/* Mod Details Modal */}
      {selectedModId && (
        <Modal
          isOpen={true}
          onClose={closeModDetails}
          title={modDetails?.name || 'Loading...'}
          size="xl"
        >
          {loadingDetails ? (
            <div className="text-center py-12">
              <div className="inline-block w-12 h-12 border-4 border-adventure-500/30 border-t-adventure-500 rounded-full animate-spin"></div>
              <p className="mt-4 text-adventure-300 font-medium">Loading mod details...</p>
            </div>
          ) : modDetails ? (
            <div className="space-y-6">
              {/* Header with Logo */}
              <div className="flex items-start gap-4">
                {modDetails.logo_url ? (
                  <img
                    src={modDetails.logo_url}
                    alt={modDetails.name}
                    className="w-24 h-24 rounded-xl object-cover shadow-lg"
                  />
                ) : (
                  <div className="w-24 h-24 flex-shrink-0 rounded-xl bg-gradient-to-br from-adventure-600 to-adventure-700 flex items-center justify-center text-5xl shadow-lg">
                    📦
                  </div>
                )}
                <div className="flex-1">
                  <h3 className="text-2xl font-bold text-white mb-2">{modDetails.name}</h3>
                  <p className="text-adventure-300 mb-3">by {modDetails.authors.join(', ')}</p>
                  <div className="flex flex-wrap gap-2">
                    <Badge variant="info">
                      ⬇️ {modDetails.download_count.toLocaleString()} downloads
                    </Badge>
                    <Badge variant="adventure">
                      📅 Updated {new Date(modDetails.date_modified).toLocaleDateString()}
                    </Badge>
                    {isModInstalled(modDetails.id) && (
                      <Badge variant="success">✓ Installed</Badge>
                    )}
                  </div>
                </div>
              </div>

              {/* Categories */}
              {modDetails.categories.length > 0 && (
                <div>
                  <h4 className="text-lg font-semibold text-white mb-2">Categories</h4>
                  <div className="flex flex-wrap gap-2">
                    {modDetails.categories.map((category) => (
                      <Badge key={category} variant="adventure">{category}</Badge>
                    ))}
                  </div>
                </div>
              )}

              {/* Description */}
              <div>
                <h4 className="text-lg font-semibold text-white mb-2">Description</h4>
                <div className="text-stone-300 leading-relaxed prose prose-invert prose-sm max-w-none">
                  {stripHtml(modDetails.description) || modDetails.summary}
                </div>
              </div>

              {/* Screenshots */}
              {modDetails.screenshots.length > 0 && (
                <div>
                  <h4 className="text-lg font-semibold text-white mb-3">Screenshots</h4>
                  <div className="grid grid-cols-2 gap-3">
                    {modDetails.screenshots.slice(0, 4).map((screenshot, idx) => (
                      <a
                        key={idx}
                        href={screenshot.url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="block rounded-lg overflow-hidden border border-adventure-500/20 hover:border-adventure-400/50 transition-colors"
                      >
                        <img
                          src={screenshot.thumbnail_url}
                          alt={screenshot.title || `Screenshot ${idx + 1}`}
                          className="w-full h-32 object-cover"
                        />
                      </a>
                    ))}
                  </div>
                </div>
              )}

              {/* Latest Files */}
              {modDetails.latest_files.length > 0 && (
                <div>
                  <h4 className="text-lg font-semibold text-white mb-3">Latest Files</h4>
                  <div className="space-y-2">
                    {modDetails.latest_files.slice(0, 3).map((file) => (
                      <div
                        key={file.id}
                        className="p-3 bg-white/5 rounded-lg border border-adventure-500/20"
                      >
                        <p className="text-white font-medium mb-1">{file.display_name}</p>
                        <div className="flex items-center gap-4 text-xs text-adventure-300">
                          <span>📦 {formatFileSize(file.file_length)}</span>
                          <span>⬇️ {file.download_count.toLocaleString()}</span>
                          <span>📅 {new Date(file.file_date).toLocaleDateString()}</span>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Links */}
              {(modDetails.links.website_url || modDetails.links.wiki_url || modDetails.links.source_url || modDetails.links.issues_url) && (
                <div>
                  <h4 className="text-lg font-semibold text-white mb-3">Links</h4>
                  <div className="flex flex-wrap gap-2">
                    {modDetails.links.website_url && (
                      <a
                        href={modDetails.links.website_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="px-4 py-2 bg-adventure-600/20 border border-adventure-500/40 hover:bg-adventure-600/30 text-stone-300 hover:text-white rounded-lg text-sm font-medium transition-colors"
                      >
                        🌐 Website
                      </a>
                    )}
                    {modDetails.links.wiki_url && (
                      <a
                        href={modDetails.links.wiki_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="px-4 py-2 bg-adventure-600/20 border border-adventure-500/40 hover:bg-adventure-600/30 text-stone-300 hover:text-white rounded-lg text-sm font-medium transition-colors"
                      >
                        📖 Wiki
                      </a>
                    )}
                    {modDetails.links.source_url && (
                      <a
                        href={modDetails.links.source_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="px-4 py-2 bg-adventure-600/20 border border-adventure-500/40 hover:bg-adventure-600/30 text-stone-300 hover:text-white rounded-lg text-sm font-medium transition-colors"
                      >
                        💻 Source
                      </a>
                    )}
                    {modDetails.links.issues_url && (
                      <a
                        href={modDetails.links.issues_url}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="px-4 py-2 bg-adventure-600/20 border border-adventure-500/40 hover:bg-adventure-600/30 text-stone-300 hover:text-white rounded-lg text-sm font-medium transition-colors"
                      >
                        🐛 Issues
                      </a>
                    )}
                  </div>
                </div>
              )}

              {/* Metadata Grid */}
              <div className="grid grid-cols-2 gap-4 p-4 bg-white/5 rounded-lg border border-adventure-500/20">
                <div>
                  <p className="text-sm text-adventure-300 mb-1">Project ID</p>
                  <p className="text-white font-semibold">{modDetails.id}</p>
                </div>
                <div>
                  <p className="text-sm text-adventure-300 mb-1">Author(s)</p>
                  <p className="text-white font-semibold">{modDetails.authors.join(', ')}</p>
                </div>
                <div>
                  <p className="text-sm text-adventure-300 mb-1">Total Downloads</p>
                  <p className="text-white font-semibold">{modDetails.download_count.toLocaleString()}</p>
                </div>
                <div>
                  <p className="text-sm text-adventure-300 mb-1">Last Updated</p>
                  <p className="text-white font-semibold">{new Date(modDetails.date_modified).toLocaleDateString()}</p>
                </div>
                <div>
                  <p className="text-sm text-adventure-300 mb-1">Created</p>
                  <p className="text-white font-semibold">{new Date(modDetails.date_created).toLocaleDateString()}</p>
                </div>
                <div>
                  <p className="text-sm text-adventure-300 mb-1">Released</p>
                  <p className="text-white font-semibold">{new Date(modDetails.date_released).toLocaleDateString()}</p>
                </div>
              </div>

              {/* Action */}
              {!isModInstalled(modDetails.id) && (
                <button
                  onClick={() => {
                    installMod(modDetails);
                    closeModDetails();
                  }}
                  disabled={installing === modDetails.id}
                  className="w-full px-6 py-3 bg-adventure-600 hover:bg-adventure-700 disabled:bg-gray-600 text-white rounded-lg font-medium transition-colors shadow-lg shadow-adventure-500/30"
                >
                  {installing === modDetails.id ? 'Installing...' : '+ Install This Mod'}
                </button>
              )}
            </div>
          ) : null}
        </Modal>
      )}
    </div>
  );
}
