import { useState, useCallback, useEffect, useMemo } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  Search,
  Loader2,
  Download,
  Check,
  X,
  AlertCircle,
  ExternalLink,
  Sparkles,
  Shield,
  Eye,
  EyeOff,
  ChevronDown,
  ChevronUp,
  Plug,
  Trash2,
  Globe,
  RefreshCw,
  CloudDownload,
  Zap,
  Wand2,
} from "lucide-react";
import {
  getMcpCatalog,
  detectRuntimes,
  installMcpFromCatalog,
  installMcpEntry,
  uninstallMcpServer,
  listMcpServers,
  connectMcpServer,
  disconnectMcpServer,
  syncMcpRegistry,
  searchMcpRegistry,
  getRegistryStatus,
  getRuntimeBootstrapStatus,
  installRuntime,
  bootstrapAllRuntimes,
  recommendMcpTools,
} from "../lib/tauri";
import type {
  CatalogEntry,
  CatalogCategory,
  RuntimeInfo,
  ConnectedServer,
  EnvVarSpec,
  RegistryStatus,
  BootstrapStatus,
  RuntimeInstallProgress,
  ToolRecommendation,
} from "../lib/types";

interface McpAppStoreProps {
  onError: (msg: string) => void;
}

export function McpAppStore({ onError }: McpAppStoreProps) {
  const [catalog, setCatalog] = useState<CatalogEntry[]>([]);
  const [categories, setCategories] = useState<CatalogCategory[]>([]);
  const [runtimes, setRuntimes] = useState<RuntimeInfo | null>(null);
  const [installedServers, setInstalledServers] = useState<ConnectedServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [activeCategory, setActiveCategory] = useState("all");
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [configEntry, setConfigEntry] = useState<CatalogEntry | null>(null);
  const [envValues, setEnvValues] = useState<Record<string, string>>({});
  const [showEnvPasswords, setShowEnvPasswords] = useState<Record<string, boolean>>({});
  const [expandedInstalled, setExpandedInstalled] = useState(true);

  // Registry state
  const [registryStatus, setRegistryStatus] = useState<RegistryStatus | null>(null);
  const [registrySyncing, setRegistrySyncing] = useState(false);
  const [registryQuery, setRegistryQuery] = useState("");
  const [registryResults, setRegistryResults] = useState<CatalogEntry[]>([]);
  const [registrySearching, setRegistrySearching] = useState(false);
  const [expandedRegistry, setExpandedRegistry] = useState(false);
  // Track registry entry IDs for install routing
  const registryEntryIds = useMemo(
    () => new Set(registryResults.map((e) => e.id)),
    [registryResults]
  );

  // Runtime Bootstrap state
  const [bootstrapStatus, setBootstrapStatus] = useState<BootstrapStatus | null>(null);
  const [bootstrapping, setBootstrapping] = useState(false);
  const [installingRuntime, setInstallingRuntime] = useState<string | null>(null);
  const [runtimeProgress, setRuntimeProgress] = useState<RuntimeInstallProgress | null>(null);

  // AI-powered tool recommendations
  const [aiQuery, setAiQuery] = useState("");
  const [recommendations, setRecommendations] = useState<ToolRecommendation[]>([]);
  const [searching, setSearching] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const [catalogData, runtimeData, servers, regStatus, bsStatus] = await Promise.all([
        getMcpCatalog(),
        detectRuntimes(),
        listMcpServers(),
        getRegistryStatus().catch(() => null),
        getRuntimeBootstrapStatus().catch(() => null),
      ]);
      setCatalog(catalogData.entries);
      setCategories(catalogData.categories);
      setRuntimes(runtimeData);
      setInstalledServers(servers);
      if (regStatus) setRegistryStatus(regStatus);
      if (bsStatus) setBootstrapStatus(bsStatus);
    } catch (e) {
      onError(String(e));
    }
  }, [onError]);

  useEffect(() => {
    refresh().finally(() => setLoading(false));
  }, [refresh]);

  // Listen for runtime install progress events
  useEffect(() => {
    const unlisten = listen<RuntimeInstallProgress>("runtime-install-progress", (event) => {
      setRuntimeProgress(event.payload);
      if (event.payload.stage === "complete" || event.payload.stage === "error") {
        setTimeout(() => {
          setRuntimeProgress(null);
          setInstallingRuntime(null);
          refresh();
        }, 1500);
      }
    });
    return () => { unlisten.then(fn => fn()); };
  }, [refresh]);

  // Handle bootstrapping all runtimes
  const handleBootstrapAll = useCallback(async () => {
    setBootstrapping(true);
    try {
      await bootstrapAllRuntimes();
      await refresh();
    } catch (e) {
      onError(String(e));
    } finally {
      setBootstrapping(false);
    }
  }, [refresh, onError]);

  // Handle installing a single runtime
  const handleInstallRuntime = useCallback(async (kind: "node" | "uv") => {
    setInstallingRuntime(kind);
    try {
      const result = await installRuntime(kind);
      if (!result.success && result.error) {
        onError(result.error);
      }
      await refresh();
    } catch (e) {
      onError(String(e));
    } finally {
      setInstallingRuntime(null);
    }
  }, [refresh, onError]);

  // AI-powered tool discovery
  const handleAiSearch = useCallback(async (query: string) => {
    setAiQuery(query);
    if (!query.trim()) {
      setRecommendations([]);
      return;
    }
    setSearching(true);
    try {
      const recs = await recommendMcpTools(query);
      setRecommendations(recs);
    } catch (e) {
      onError(String(e));
    } finally {
      setSearching(false);
    }
  }, [onError]);

  // Debounced AI search
  useEffect(() => {
    if (!aiQuery.trim()) { setRecommendations([]); return; }
    const timer = setTimeout(() => handleAiSearch(aiQuery), 400);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [aiQuery]);

  // Get installed server names for quick lookup
  const installedNames = useMemo(
    () => new Set(installedServers.map((s) => s.name)),
    [installedServers]
  );

  // Filter catalog entries
  const filteredEntries = useMemo(() => {
    let entries = catalog;

    // Category filter
    if (activeCategory !== "all") {
      entries = entries.filter((e) => e.category === activeCategory);
    }

    // Search filter
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      entries = entries.filter(
        (e) =>
          e.name.toLowerCase().includes(q) ||
          e.description.toLowerCase().includes(q) ||
          e.tags.some((t) => t.includes(q))
      );
    }

    // Sort by popularity
    return entries.sort((a, b) => a.popularity - b.popularity);
  }, [catalog, activeCategory, searchQuery]);

  // Check if runtime is available for an entry
  const canInstall = useCallback(
    (entry: CatalogEntry): boolean => {
      if (!runtimes) return false;
      if (entry.runtime === "node") {
        return entry.command === "npx" ? runtimes.has_npx : runtimes.has_node;
      }
      if (entry.runtime === "python") {
        return entry.command === "uvx" ? runtimes.has_uvx : runtimes.has_python;
      }
      return true;
    },
    [runtimes]
  );

  // Handle one-click install (no env vars needed)
  const handleInstall = useCallback(
    async (entry: CatalogEntry, fromRegistry = false) => {
      // If entry requires env vars, show config dialog
      if (entry.required_env.length > 0) {
        setConfigEntry(entry);
        setEnvValues({});
        return;
      }

      setInstallingId(entry.id);
      try {
        if (fromRegistry || registryEntryIds.has(entry.id)) {
          await installMcpEntry(entry, {});
        } else {
          await installMcpFromCatalog(entry.id, {});
        }
        await refresh();
      } catch (e) {
        onError(String(e));
      } finally {
        setInstallingId(null);
      }
    },
    [refresh, onError, registryEntryIds]
  );

  // Handle install with env vars configured
  const handleInstallWithConfig = useCallback(async () => {
    if (!configEntry) return;

    // Validate required vars
    for (const spec of configEntry.required_env) {
      if (spec.required && !envValues[spec.name]?.trim()) {
        onError(`"${spec.label}" is required`);
        return;
      }
    }

    setInstallingId(configEntry.id);
    const isReg = registryEntryIds.has(configEntry.id);
    setConfigEntry(null);
    try {
      if (isReg) {
        await installMcpEntry(configEntry, envValues);
      } else {
        await installMcpFromCatalog(configEntry.id, envValues);
      }
      await refresh();
    } catch (e) {
      onError(String(e));
    } finally {
      setInstallingId(null);
      setEnvValues({});
    }
  }, [configEntry, envValues, refresh, onError, registryEntryIds]);

  // Handle uninstall
  const handleUninstall = useCallback(
    async (name: string) => {
      try {
        await uninstallMcpServer(name);
        await refresh();
      } catch (e) {
        onError(String(e));
      }
    },
    [refresh, onError]
  );

  // Handle reconnect
  const handleReconnect = useCallback(
    async (name: string) => {
      try {
        await connectMcpServer(name);
        await refresh();
      } catch (e) {
        onError(String(e));
      }
    },
    [refresh, onError]
  );

  // Handle disconnect
  const handleDisconnect = useCallback(
    async (name: string) => {
      try {
        await disconnectMcpServer(name);
        await refresh();
      } catch (e) {
        onError(String(e));
      }
    },
    [refresh, onError]
  );

  // --- Registry Handlers ---

  // Sync the official MCP Registry
  const handleRegistrySync = useCallback(async () => {
    setRegistrySyncing(true);
    try {
      const result = await syncMcpRegistry();
      if (result.success) {
        const status = await getRegistryStatus();
        setRegistryStatus(status);
        setExpandedRegistry(true);
      } else {
        onError(result.error || "Registry sync failed");
      }
    } catch (e) {
      onError(String(e));
    } finally {
      setRegistrySyncing(false);
    }
  }, [onError]);

  // Search the registry cache
  const handleRegistrySearch = useCallback(
    async (query: string) => {
      setRegistryQuery(query);
      if (!query.trim()) {
        setRegistryResults([]);
        return;
      }
      setRegistrySearching(true);
      try {
        const results = await searchMcpRegistry(query, 30);
        setRegistryResults(results);
      } catch (e) {
        onError(String(e));
      } finally {
        setRegistrySearching(false);
      }
    },
    [onError]
  );

  // Debounced registry search
  useEffect(() => {
    if (!registryQuery.trim()) {
      setRegistryResults([]);
      return;
    }
    const timer = setTimeout(() => {
      handleRegistrySearch(registryQuery);
    }, 300);
    return () => clearTimeout(timer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [registryQuery]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-5 h-5 animate-spin text-ghost-text-dim" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* ⚡ Runtime Bootstrap Panel */}
      {bootstrapStatus && !bootstrapStatus.ready_for_defaults && (
        <div className="px-3 py-3 bg-gradient-to-r from-ghost-accent/10 to-purple-500/10 border border-ghost-accent/20 rounded-xl space-y-3">
          <div className="flex items-center gap-2">
            <Zap className="w-4 h-4 text-ghost-accent" />
            <span className="text-sm font-medium text-ghost-text">
              Runtime Setup
            </span>
            <span className="text-[10px] text-ghost-text-dim ml-auto">
              Ghost can auto-install these for you
            </span>
          </div>

          {/* Runtime status pills */}
          <div className="flex flex-wrap gap-2">
            {bootstrapStatus.runtimes.map((rt) => (
              <div
                key={rt.kind}
                className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-lg text-xs border transition-all ${
                  rt.installed
                    ? "bg-green-500/10 border-green-500/20 text-green-400"
                    : rt.can_auto_install
                    ? "bg-ghost-surface border-ghost-border text-ghost-text-dim hover:border-ghost-accent/30"
                    : "bg-ghost-surface border-ghost-border text-ghost-text-dim/50"
                }`}
              >
                <span
                  className={`w-1.5 h-1.5 rounded-full ${
                    rt.installed ? "bg-green-400" : "bg-ghost-text-dim/30"
                  }`}
                />
                <span className="font-medium capitalize">{rt.kind}</span>
                {rt.installed ? (
                  <span className="text-[10px] opacity-70">
                    {rt.version ? rt.version.split(" ")[0] : "✓"}
                    {rt.managed && " (Ghost)"}
                  </span>
                ) : rt.can_auto_install ? (
                  <button
                    onClick={() => handleInstallRuntime(rt.kind as "node" | "uv")}
                    disabled={!!installingRuntime}
                    className="ml-1 px-1.5 py-0.5 bg-ghost-accent/20 text-ghost-accent rounded text-[10px] hover:bg-ghost-accent/30 transition-colors disabled:opacity-50"
                  >
                    {installingRuntime === rt.kind ? (
                      <Loader2 className="w-3 h-3 animate-spin" />
                    ) : (
                      "Install"
                    )}
                  </button>
                ) : (
                  <span className="text-[10px] opacity-50">manual</span>
                )}
              </div>
            ))}
          </div>

          {/* Progress bar during installation */}
          {runtimeProgress && (
            <div className="space-y-1">
              <div className="flex items-center justify-between text-[10px] text-ghost-text-dim">
                <span>{runtimeProgress.message}</span>
                <span>{runtimeProgress.percent >= 0 ? `${Math.round(runtimeProgress.percent)}%` : ""}</span>
              </div>
              <div className="h-1 bg-ghost-surface rounded-full overflow-hidden">
                <div
                  className="h-full bg-ghost-accent rounded-full transition-all duration-300"
                  style={{ width: `${Math.max(0, runtimeProgress.percent)}%` }}
                />
              </div>
            </div>
          )}

          {/* Install All button */}
          {bootstrapStatus.missing_installable.length > 0 && !installingRuntime && (
            <button
              onClick={handleBootstrapAll}
              disabled={bootstrapping}
              className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-ghost-accent/20 text-ghost-accent text-xs font-medium rounded-lg hover:bg-ghost-accent/30 transition-colors disabled:opacity-50"
            >
              {bootstrapping ? (
                <>
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  Installing runtimes...
                </>
              ) : (
                <>
                  <Download className="w-3.5 h-3.5" />
                  Install All Missing Runtimes ({bootstrapStatus.missing_installable.length})
                </>
              )}
            </button>
          )}
        </div>
      )}

      {/* 🔮 AI Tool Discovery */}
      <div className="space-y-2">
        <div className="relative">
          <Wand2 className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-purple-400" />
          <input
            type="text"
            value={aiQuery}
            onChange={(e) => setAiQuery(e.target.value)}
            placeholder="Describe what you need... e.g. 'manage GitHub repos' or 'query databases'"
            className="w-full pl-8 pr-3 py-2 bg-ghost-surface border border-ghost-border rounded-lg text-xs text-ghost-text placeholder-ghost-text-dim/50 focus:outline-none focus:border-purple-500/50"
          />
          {searching && (
            <Loader2 className="absolute right-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 animate-spin text-purple-400" />
          )}
        </div>
        {recommendations.length > 0 && (
          <div className="space-y-1">
            <span className="text-[10px] text-purple-400/70 uppercase tracking-wider font-medium">
              Recommended Tools
            </span>
            {recommendations.slice(0, 5).map((rec) => (
              <div
                key={rec.catalog_id ?? rec.name}
                className="flex items-center gap-2 px-2.5 py-2 bg-ghost-surface border border-purple-500/10 rounded-lg"
              >
                <Sparkles className="w-3.5 h-3.5 text-purple-400 shrink-0" />
                <div className="flex-1 min-w-0">
                  <span className="text-xs text-ghost-text font-medium block truncate">
                    {rec.name}
                  </span>
                  <span className="text-[10px] text-ghost-text-dim truncate block">
                    {rec.reason.length > 80 ? rec.reason.substring(0, 80) + "..." : rec.reason}
                  </span>
                </div>
                <div className="flex items-center gap-1 shrink-0">
                  {rec.missing_runtimes.length > 0 && (
                    <span className="text-[9px] text-amber-400 px-1.5 py-0.5 bg-amber-400/10 rounded">
                      needs {rec.missing_runtimes.join(", ")}
                    </span>
                  )}
                  {rec.installable && rec.catalog_id && (
                    <button
                      onClick={() => {
                        const entry = catalog.find((e) => e.id === rec.catalog_id);
                        if (entry) handleInstall(entry);
                      }}
                      className="px-2 py-1 bg-ghost-accent/20 text-ghost-accent text-[10px] rounded hover:bg-ghost-accent/30 transition-colors"
                    >
                      Install
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Installed Servers Section */}
      {installedServers.length > 0 && (
        <div>
          <button
            onClick={() => setExpandedInstalled(!expandedInstalled)}
            className="flex items-center gap-2 w-full text-left mb-2"
          >
            <Plug className="w-4 h-4 text-ghost-accent" />
            <span className="text-sm font-medium text-ghost-text flex-1">
              Installed ({installedServers.length})
            </span>
            {expandedInstalled ? (
              <ChevronUp className="w-4 h-4 text-ghost-text-dim" />
            ) : (
              <ChevronDown className="w-4 h-4 text-ghost-text-dim" />
            )}
          </button>
          {expandedInstalled && (
            <div className="space-y-1.5">
              {installedServers.map((server) => (
                <div
                  key={server.name}
                  className="flex items-center gap-2.5 px-3 py-2 bg-ghost-bg rounded-lg border border-ghost-border"
                >
                  <span
                    className={`w-2 h-2 rounded-full shrink-0 ${
                      server.connected ? "bg-green-400" : "bg-ghost-text-dim/30"
                    }`}
                  />
                  <div className="flex-1 min-w-0">
                    <span className="text-sm text-ghost-text truncate block">
                      {server.name}
                    </span>
                    {server.connected && server.tools.length > 0 && (
                      <span className="text-[10px] text-ghost-text-dim">
                        {server.tools.length} tool
                        {server.tools.length !== 1 ? "s" : ""}
                      </span>
                    )}
                    {server.error && (
                      <span className="text-[10px] text-ghost-danger flex items-center gap-0.5">
                        <AlertCircle className="w-2.5 h-2.5" />
                        {server.error.length > 50
                          ? server.error.substring(0, 50) + "..."
                          : server.error}
                      </span>
                    )}
                  </div>
                  <div className="flex items-center gap-1 shrink-0">
                    {server.connected ? (
                      <button
                        onClick={() => handleDisconnect(server.name)}
                        className="p-1 rounded text-ghost-text-dim hover:text-amber-400 hover:bg-amber-400/10 transition-all"
                        title="Disconnect"
                      >
                        <X className="w-3 h-3" />
                      </button>
                    ) : (
                      <button
                        onClick={() => handleReconnect(server.name)}
                        className="p-1 rounded text-ghost-text-dim hover:text-ghost-accent hover:bg-ghost-accent/10 transition-all"
                        title="Reconnect"
                      >
                        <Plug className="w-3 h-3" />
                      </button>
                    )}
                    <button
                      onClick={() => handleUninstall(server.name)}
                      className="p-1 rounded text-ghost-text-dim hover:text-ghost-danger hover:bg-ghost-danger/10 transition-all"
                      title="Uninstall"
                    >
                      <Trash2 className="w-3 h-3" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Search Bar */}
      <div className="relative">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-ghost-text-dim/40" />
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          placeholder="Search tools..."
          className="w-full pl-9 pr-3 py-2 bg-ghost-bg border border-ghost-border rounded-xl text-sm text-ghost-text outline-none focus:border-ghost-accent/40 placeholder:text-ghost-text-dim/30 transition-colors"
        />
      </div>

      {/* Category Chips */}
      <div className="flex gap-1.5 overflow-x-auto pb-1 -mx-1 px-1 scrollbar-hide">
        {categories.map((cat) => (
          <button
            key={cat.id}
            onClick={() => setActiveCategory(cat.id)}
            className={`shrink-0 px-2.5 py-1 rounded-lg text-xs font-medium transition-all ${
              activeCategory === cat.id
                ? "bg-ghost-accent/15 text-ghost-accent border border-ghost-accent/30"
                : "bg-ghost-bg text-ghost-text-dim border border-ghost-border hover:border-ghost-accent/20 hover:text-ghost-text"
            }`}
          >
            <span className="mr-1">{cat.icon}</span>
            {cat.name}
          </button>
        ))}
      </div>

      {/* Catalog Grid */}
      <div className="space-y-2">
        {filteredEntries.length === 0 ? (
          <div className="py-8 text-center">
            <Search className="w-8 h-8 text-ghost-text-dim/20 mx-auto mb-2" />
            <p className="text-sm text-ghost-text-dim">No tools found</p>
            <p className="text-xs text-ghost-text-dim/50 mt-0.5">
              Try a different search or category
            </p>
          </div>
        ) : (
          filteredEntries.map((entry) => {
            const installed = installedNames.has(entry.name);
            const installing = installingId === entry.id;
            const installable = canInstall(entry);

            return (
              <div
                key={entry.id}
                className="flex items-center gap-3 px-3 py-2.5 bg-ghost-bg rounded-xl border border-ghost-border hover:border-ghost-accent/20 transition-all group"
              >
                {/* Icon */}
                <span className="text-xl shrink-0 w-8 text-center">{entry.icon}</span>

                {/* Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-1.5">
                    <span className="text-sm font-medium text-ghost-text truncate">
                      {entry.name}
                    </span>
                    {entry.official && (
                      <span title="Official">
                        <Shield className="w-3 h-3 text-ghost-accent shrink-0" />
                      </span>
                    )}
                    {entry.is_mcp_app && (
                      <span title="MCP App (has UI)">
                        <Sparkles className="w-3 h-3 text-purple-400 shrink-0" />
                      </span>
                    )}
                  </div>
                  <p className="text-[11px] text-ghost-text-dim/70 truncate mt-0.5">
                    {entry.description}
                  </p>
                  {entry.required_env.length > 0 && !installed && (
                    <p className="text-[10px] text-amber-400/70 mt-0.5">
                      Requires configuration
                    </p>
                  )}
                </div>

                {/* Action Button */}
                <div className="shrink-0">
                  {installed ? (
                    <span className="flex items-center gap-1 px-2 py-1 text-xs text-green-400 bg-green-400/10 rounded-lg">
                      <Check className="w-3 h-3" />
                      Installed
                    </span>
                  ) : installing ? (
                    <span className="flex items-center gap-1 px-2 py-1 text-xs text-ghost-accent">
                      <Loader2 className="w-3 h-3 animate-spin" />
                    </span>
                  ) : !installable ? (
                    <span
                      className="px-2 py-1 text-[10px] text-ghost-text-dim/40 rounded-lg"
                      title={`Requires ${entry.runtime === "node" ? "Node.js" : "Python"}`}
                    >
                      {entry.runtime === "node" ? "Need Node.js" : "Need Python"}
                    </span>
                  ) : (
                    <button
                      onClick={() => handleInstall(entry)}
                      className="flex items-center gap-1 px-2.5 py-1 text-xs font-medium text-ghost-accent bg-ghost-accent/10 hover:bg-ghost-accent/20 rounded-lg transition-all"
                    >
                      <Download className="w-3 h-3" />
                      Install
                    </button>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>

      {/* ─── Official MCP Registry Section ─── */}
      <div className="pt-3 border-t border-ghost-border/50">
        <button
          onClick={() => setExpandedRegistry(!expandedRegistry)}
          className="flex items-center gap-2 w-full text-left mb-2"
        >
          <Globe className="w-4 h-4 text-ghost-accent" />
          <span className="text-sm font-medium text-ghost-text flex-1">
            Official MCP Registry
            {registryStatus?.meta && (
              <span className="text-[10px] text-ghost-text-dim/60 font-normal ml-1.5">
                ({registryStatus.meta.installable_count.toLocaleString()} installable servers)
              </span>
            )}
          </span>
          {expandedRegistry ? (
            <ChevronUp className="w-4 h-4 text-ghost-text-dim" />
          ) : (
            <ChevronDown className="w-4 h-4 text-ghost-text-dim" />
          )}
        </button>

        {expandedRegistry && (
          <div className="space-y-3">
            {/* Sync Status & Button */}
            {!registryStatus?.synced ? (
              <div className="flex flex-col items-center gap-3 py-6 px-4 bg-ghost-bg/50 rounded-xl border border-ghost-border border-dashed">
                <CloudDownload className="w-8 h-8 text-ghost-text-dim/30" />
                <div className="text-center">
                  <p className="text-sm text-ghost-text">
                    Browse 6,000+ MCP Servers
                  </p>
                  <p className="text-[11px] text-ghost-text-dim/60 mt-0.5">
                    Sync the official registry to discover and install any MCP tool
                  </p>
                </div>
                <button
                  onClick={handleRegistrySync}
                  disabled={registrySyncing}
                  className="flex items-center gap-2 px-4 py-2 bg-ghost-accent/10 text-ghost-accent rounded-xl text-sm font-medium hover:bg-ghost-accent/20 disabled:opacity-50 transition-all"
                >
                  {registrySyncing ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      Syncing...
                    </>
                  ) : (
                    <>
                      <Download className="w-4 h-4" />
                      Sync Registry
                    </>
                  )}
                </button>
              </div>
            ) : (
              <>
                {/* Registry Search */}
                <div className="flex gap-2">
                  <div className="relative flex-1">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-ghost-text-dim/40" />
                    <input
                      type="text"
                      value={registryQuery}
                      onChange={(e) => setRegistryQuery(e.target.value)}
                      placeholder="Search 6,000+ servers (e.g., slack, database, stripe)..."
                      className="w-full pl-9 pr-3 py-2 bg-ghost-bg border border-ghost-border rounded-xl text-sm text-ghost-text outline-none focus:border-ghost-accent/40 placeholder:text-ghost-text-dim/30 transition-colors"
                    />
                    {registrySearching && (
                      <Loader2 className="absolute right-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 animate-spin text-ghost-accent" />
                    )}
                  </div>
                  <button
                    onClick={handleRegistrySync}
                    disabled={registrySyncing}
                    className="p-2 rounded-xl border border-ghost-border text-ghost-text-dim hover:text-ghost-accent hover:border-ghost-accent/30 transition-all disabled:opacity-50"
                    title={registryStatus.fresh ? "Registry is up to date" : "Refresh registry cache"}
                  >
                    <RefreshCw className={`w-4 h-4 ${registrySyncing ? "animate-spin" : ""}`} />
                  </button>
                </div>

                {/* Cache Info */}
                {registryStatus.meta && (
                  <p className="text-[10px] text-ghost-text-dim/40 -mt-1">
                    {registryStatus.meta.total_servers.toLocaleString()} servers cached
                    {registryStatus.meta.last_sync && (
                      <> · Last synced {new Date(registryStatus.meta.last_sync).toLocaleDateString()}</>
                    )}
                    {!registryStatus.fresh && (
                      <span className="text-amber-400/60"> · Cache expired</span>
                    )}
                  </p>
                )}

                {/* Registry Results */}
                {registryQuery.trim() && (
                  <div className="space-y-2">
                    {registryResults.length === 0 && !registrySearching ? (
                      <div className="py-6 text-center">
                        <Search className="w-6 h-6 text-ghost-text-dim/20 mx-auto mb-1.5" />
                        <p className="text-xs text-ghost-text-dim">
                          No servers found for "{registryQuery}"
                        </p>
                      </div>
                    ) : (
                      registryResults.map((entry) => {
                        const installed = installedNames.has(entry.name);
                        const installing = installingId === entry.id;
                        const installable = canInstall(entry) || entry.transport === "http";

                        return (
                          <div
                            key={entry.id}
                            className="flex items-center gap-3 px-3 py-2.5 bg-ghost-bg rounded-xl border border-ghost-border hover:border-ghost-accent/20 transition-all group"
                          >
                            <span className="text-xl shrink-0 w-8 text-center">{entry.icon}</span>
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-1.5">
                                <span className="text-sm font-medium text-ghost-text truncate">
                                  {entry.name}
                                </span>
                                <span className="px-1.5 py-0.5 text-[9px] font-medium bg-ghost-accent/10 text-ghost-accent rounded">
                                  Registry
                                </span>
                                {entry.transport === "http" && (
                                  <span className="px-1.5 py-0.5 text-[9px] font-medium bg-purple-400/10 text-purple-400 rounded">
                                    Remote
                                  </span>
                                )}
                              </div>
                              <p className="text-[11px] text-ghost-text-dim/70 truncate mt-0.5">
                                {entry.description}
                              </p>
                              <div className="flex items-center gap-2 mt-0.5">
                                <span className="text-[10px] text-ghost-text-dim/40">
                                  {entry.runtime === "remote" ? "HTTP" : entry.runtime}
                                </span>
                                {entry.package && (
                                  <span className="text-[10px] text-ghost-text-dim/40 truncate">
                                    {entry.package}
                                  </span>
                                )}
                                {entry.required_env.length > 0 && (
                                  <span className="text-[10px] text-amber-400/70">
                                    Requires config
                                  </span>
                                )}
                              </div>
                            </div>
                            <div className="shrink-0">
                              {installed ? (
                                <span className="flex items-center gap-1 px-2 py-1 text-xs text-green-400 bg-green-400/10 rounded-lg">
                                  <Check className="w-3 h-3" />
                                  Installed
                                </span>
                              ) : installing ? (
                                <span className="flex items-center gap-1 px-2 py-1 text-xs text-ghost-accent">
                                  <Loader2 className="w-3 h-3 animate-spin" />
                                </span>
                              ) : !installable ? (
                                <span
                                  className="px-2 py-1 text-[10px] text-ghost-text-dim/40 rounded-lg"
                                  title={`Requires ${entry.runtime}`}
                                >
                                  Need {entry.runtime === "node" ? "Node.js" : entry.runtime === "python" ? "Python" : entry.runtime}
                                </span>
                              ) : (
                                <button
                                  onClick={() => handleInstall(entry, true)}
                                  className="flex items-center gap-1 px-2.5 py-1 text-xs font-medium text-ghost-accent bg-ghost-accent/10 hover:bg-ghost-accent/20 rounded-lg transition-all"
                                >
                                  <Download className="w-3 h-3" />
                                  Install
                                </button>
                              )}
                            </div>
                          </div>
                        );
                      })
                    )}
                  </div>
                )}

                {/* Empty state when no search */}
                {!registryQuery.trim() && (
                  <div className="py-4 text-center">
                    <p className="text-xs text-ghost-text-dim/50">
                      Type a search term to discover servers from the official MCP Registry
                    </p>
                  </div>
                )}
              </>
            )}
          </div>
        )}
      </div>

      {/* Configuration Dialog (for tools that need env vars) */}
      {configEntry && (
        <ConfigDialog
          entry={configEntry}
          envValues={envValues}
          showPasswords={showEnvPasswords}
          onEnvChange={(name, value) =>
            setEnvValues((prev) => ({ ...prev, [name]: value }))
          }
          onTogglePassword={(name) =>
            setShowEnvPasswords((prev) => ({ ...prev, [name]: !prev[name] }))
          }
          onInstall={handleInstallWithConfig}
          onCancel={() => {
            setConfigEntry(null);
            setEnvValues({});
          }}
          installing={installingId === configEntry.id}
        />
      )}
    </div>
  );
}

// ─── Configuration Dialog ─────────────────────────────────

interface ConfigDialogProps {
  entry: CatalogEntry;
  envValues: Record<string, string>;
  showPasswords: Record<string, boolean>;
  onEnvChange: (name: string, value: string) => void;
  onTogglePassword: (name: string) => void;
  onInstall: () => void;
  onCancel: () => void;
  installing: boolean;
}

function ConfigDialog({
  entry,
  envValues,
  showPasswords,
  onEnvChange,
  onTogglePassword,
  onInstall,
  onCancel,
  installing,
}: ConfigDialogProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <div className="bg-ghost-surface border border-ghost-border rounded-2xl shadow-2xl w-full max-w-sm mx-4 overflow-hidden">
        {/* Header */}
        <div className="flex items-center gap-3 px-5 pt-5 pb-3">
          <span className="text-2xl">{entry.icon}</span>
          <div className="flex-1">
            <h3 className="text-sm font-semibold text-ghost-text">
              Install {entry.name}
            </h3>
            <p className="text-[11px] text-ghost-text-dim mt-0.5">
              Configure required settings
            </p>
          </div>
          <button
            onClick={onCancel}
            className="p-1 rounded-lg text-ghost-text-dim hover:text-ghost-text hover:bg-ghost-surface-hover transition-all"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Config Fields */}
        <div className="px-5 pb-3 space-y-3">
          {entry.required_env.map((spec: EnvVarSpec) => (
            <div key={spec.name}>
              <label className="flex items-center gap-1 text-xs text-ghost-text-dim mb-1">
                {spec.label}
                {spec.required && <span className="text-ghost-danger">*</span>}
              </label>
              <div className="relative">
                <input
                  type={spec.sensitive && !showPasswords[spec.name] ? "password" : "text"}
                  value={envValues[spec.name] || ""}
                  onChange={(e) => onEnvChange(spec.name, e.target.value)}
                  placeholder={spec.placeholder || ""}
                  className="w-full px-3 py-2 bg-ghost-bg border border-ghost-border rounded-lg text-sm text-ghost-text outline-none focus:border-ghost-accent/50 pr-8 transition-colors"
                />
                {spec.sensitive && (
                  <button
                    type="button"
                    onClick={() => onTogglePassword(spec.name)}
                    className="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 text-ghost-text-dim/40 hover:text-ghost-text-dim transition-colors"
                  >
                    {showPasswords[spec.name] ? (
                      <EyeOff className="w-3.5 h-3.5" />
                    ) : (
                      <Eye className="w-3.5 h-3.5" />
                    )}
                  </button>
                )}
              </div>
              <p className="text-[10px] text-ghost-text-dim/50 mt-0.5">
                {spec.description}
              </p>
            </div>
          ))}
        </div>

        {/* Actions */}
        <div className="flex gap-2 px-5 pb-5 pt-2">
          <button
            onClick={onInstall}
            disabled={installing}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-ghost-accent text-ghost-bg rounded-xl text-sm font-medium hover:bg-ghost-accent/90 disabled:opacity-50 transition-all"
          >
            {installing ? (
              <Loader2 className="w-4 h-4 animate-spin" />
            ) : (
              <Download className="w-4 h-4" />
            )}
            Install & Connect
          </button>
          <button
            onClick={onCancel}
            className="px-4 py-2 bg-ghost-bg border border-ghost-border text-ghost-text-dim rounded-xl text-sm hover:bg-ghost-surface-hover transition-all"
          >
            Cancel
          </button>
        </div>

        {/* Repository Link */}
        {entry.repository && (
          <div className="px-5 pb-4 -mt-1">
            <a
              href={entry.repository}
              target="_blank"
              rel="noreferrer"
              className="inline-flex items-center gap-1 text-[10px] text-ghost-text-dim/40 hover:text-ghost-accent transition-colors"
            >
              <ExternalLink className="w-2.5 h-2.5" />
              View source
            </a>
          </div>
        )}
      </div>
    </div>
  );
}
