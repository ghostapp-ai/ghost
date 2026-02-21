import { useState, useCallback, useEffect } from "react";
import {
  X,
  Loader2,
  FolderOpen,
  ChevronRight,
  Home,
  HardDrive,
  Cpu,
  Download,
  Check,
  Star,
  Trash2,
  Plus,
  ArrowLeft,
  Cloud,
  Monitor,
  Zap,
  Plug,
} from "lucide-react";
import {
  getSettings,
  saveSettings,
  startWatcher,
  getAvailableModels,
  getHardwareInfo,
  chatSwitchModel,
  chatLoadModel,
  listDirectory,
  getRootDirectories,
  addWatchDirectory,
  removeWatchDirectory,
  getMcpServerStatus,
} from "../lib/tauri";
import { usePlatform } from "../hooks/usePlatform";
import { useUpdater } from "../hooks/useUpdater";
import { CheckUpdateButton } from "./UpdateNotification";
import { McpAppStore } from "./McpAppStore";
import type {
  Settings as SettingsType,
  ModelInfo,
  HardwareInfo,
  FsEntry,
  McpServerStatus,
} from "../lib/types";

interface SettingsProps {
  onClose: () => void;
}

type Tab = "general" | "models" | "directories" | "mcp";

export function Settings({ onClose }: SettingsProps) {
  const { isMobile } = usePlatform();
  const [tab, setTab] = useState<Tab>("general");
  const [settings, setSettings] = useState<SettingsType | null>(null);
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [hardware, setHardware] = useState<HardwareInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load all data on mount
  useEffect(() => {
    Promise.all([
      getSettings(),
      getAvailableModels(),
      getHardwareInfo(),
    ])
      .then(([s, m, h]) => {
        setSettings(s);
        setModels(m);
        setHardware(h);
      })
      .catch((e) => setError(String(e)))
      .finally(() => setLoading(false));
  }, []);

  const handleSave = useCallback(async (newSettings: SettingsType) => {
    try {
      await saveSettings(newSettings);
      setSettings(newSettings);
      if (newSettings.watched_directories.length > 0) {
        await startWatcher(newSettings.watched_directories).catch(() => {});
      }
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  if (loading) {
    return (
      <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center">
        <div className="bg-ghost-surface border border-ghost-border rounded-2xl p-8 shadow-2xl">
          <Loader2 className="w-6 h-6 animate-spin text-ghost-accent mx-auto" />
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-0 md:p-4">
      <div className={`bg-ghost-surface flex flex-col overflow-hidden ${
        isMobile
          ? "w-full h-full"
          : "border border-ghost-border rounded-2xl w-full max-w-2xl shadow-2xl max-h-[85vh]"
      }`}>
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-ghost-border shrink-0 pt-safe">
          <h2 className="text-lg font-semibold text-ghost-text">Settings</h2>
          <button
            onClick={onClose}
            className={`rounded-lg text-ghost-text-dim hover:text-ghost-text hover:bg-ghost-surface-hover transition-all ${
              isMobile ? "p-2.5" : "p-1.5"
            }`}
            aria-label="Close settings"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tabs — horizontally scrollable on mobile */}
        <div className="flex border-b border-ghost-border shrink-0 px-6 overflow-x-auto scrollbar-none">
          <TabButton
            active={tab === "general"}
            onClick={() => setTab("general")}
            icon={<Monitor className="w-3.5 h-3.5" />}
            label="General"
          />
          <TabButton
            active={tab === "models"}
            onClick={() => setTab("models")}
            icon={<Cpu className="w-3.5 h-3.5" />}
            label="AI Models"
          />
          <TabButton
            active={tab === "directories"}
            onClick={() => setTab("directories")}
            icon={<FolderOpen className="w-3.5 h-3.5" />}
            label="Directories"
            hide={isMobile}
          />
          <TabButton
            active={tab === "mcp"}
            onClick={() => setTab("mcp")}
            icon={<Plug className="w-3.5 h-3.5" />}
            label="MCP"
          />
        </div>

        {/* Content — scrollable */}
        <div className="flex-1 overflow-y-auto p-6">
          {error && (
            <div className="mb-4 px-4 py-3 bg-ghost-danger/10 border border-ghost-danger/20 rounded-xl text-sm text-ghost-danger">
              {error}
              <button onClick={() => setError(null)} className="ml-2 underline text-xs">
                dismiss
              </button>
            </div>
          )}

          {saved && (
            <div className="mb-4 px-4 py-3 bg-ghost-success/10 border border-ghost-success/20 rounded-xl text-sm text-ghost-success flex items-center gap-2">
              <Check className="w-4 h-4" />
              Settings saved
            </div>
          )}

          {tab === "general" && settings && (
            <GeneralTab settings={settings} hardware={hardware} onSave={handleSave} />
          )}
          {tab === "models" && settings && (
            <ModelsTab
              models={models}
              hardware={hardware}
              settings={settings}
              onModelSwitch={(modelId) => {
                chatSwitchModel(modelId)
                  .then(() => getAvailableModels())
                  .then(setModels)
                  .catch((e) => setError(String(e)));
              }}
              onModelDownload={() => {
                chatLoadModel()
                  .then(() => getAvailableModels())
                  .then(setModels)
                  .catch((e) => setError(String(e)));
              }}
            />
          )}
          {tab === "directories" && settings && (
            <DirectoriesTab
              settings={settings}
              onSave={handleSave}
            />
          )}
          {tab === "mcp" && settings && (
            <McpTab onError={setError} />
          )}
        </div>
      </div>
    </div>
  );
}

// ─── Tab Button ────────────────────────────────────────────

function TabButton({
  active,
  onClick,
  icon,
  label,
  hide = false,
}: {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
  hide?: boolean;
}) {
  if (hide) return null;
  return (
    <button
      onClick={onClick}
      className={`flex items-center gap-1.5 px-4 py-3 text-sm font-medium border-b-2 transition-all ${
        active
          ? "text-ghost-accent border-ghost-accent"
          : "text-ghost-text-dim border-transparent hover:text-ghost-text hover:border-ghost-border"
      }`}
    >
      {icon}
      {label}
    </button>
  );
}

// ─── General Tab ───────────────────────────────────────────

function GeneralTab({
  settings,
  hardware,
  onSave,
}: {
  settings: SettingsType;
  hardware: HardwareInfo | null;
  onSave: (s: SettingsType) => void;
}) {
  const [shortcut, setShortcut] = useState(settings.shortcut);
  const [maxTokens, setMaxTokens] = useState(settings.chat_max_tokens);
  const [temperature, setTemperature] = useState(settings.chat_temperature);
  const { isDesktop } = usePlatform();
  const updater = useUpdater(false); // no auto-check, manual only

  return (
    <div className="space-y-6">
      {/* Hardware Info */}
      {hardware && (
        <Section title="Hardware" icon={<Zap className="w-4 h-4" />}>
          <div className="grid grid-cols-2 gap-3">
            <InfoCard label="CPU Cores" value={`${hardware.cpu_cores}`} />
            <InfoCard label="RAM" value={`${(hardware.total_ram_mb / 1024).toFixed(1)} GB`} />
            <InfoCard
              label="GPU"
              value={hardware.gpu_backend ?? "CPU only"}
            />
            <InfoCard
              label="SIMD"
              value={
                hardware.has_avx2
                  ? "AVX2"
                  : hardware.has_neon
                  ? "NEON"
                  : "Basic"
              }
            />
          </div>
        </Section>
      )}

      {/* Global Shortcut */}
      <Section title="Keyboard Shortcut" icon={<Monitor className="w-4 h-4" />}>
        <div className="flex items-center gap-3">
          <input
            type="text"
            value={shortcut}
            onChange={(e) => setShortcut(e.target.value)}
            className="flex-1 px-3 py-2 bg-ghost-bg border border-ghost-border rounded-lg text-sm text-ghost-text outline-none focus:border-ghost-accent/50"
            placeholder="CmdOrCtrl+Space"
          />
          <button
            onClick={() => onSave({ ...settings, shortcut })}
            className="px-4 py-2 bg-ghost-accent/20 text-ghost-accent rounded-lg text-sm font-medium hover:bg-ghost-accent/30 transition-all"
          >
            Apply
          </button>
        </div>
        <p className="text-xs text-ghost-text-dim/40 mt-1">
          Summon Ghost from anywhere on your desktop
        </p>
      </Section>

      {/* Chat Settings */}
      <Section title="Chat Settings" icon={<Cpu className="w-4 h-4" />}>
        <div className="space-y-4">
          <div>
            <label className="text-xs text-ghost-text-dim mb-1 block">
              Max Tokens: {maxTokens}
            </label>
            <input
              type="range"
              min={64}
              max={2048}
              step={64}
              value={maxTokens}
              onChange={(e) => setMaxTokens(Number(e.target.value))}
              onMouseUp={() => onSave({ ...settings, chat_max_tokens: maxTokens })}
              className="w-full accent-ghost-accent"
            />
            <div className="flex justify-between text-[10px] text-ghost-text-dim/30">
              <span>64</span>
              <span>2048</span>
            </div>
          </div>

          <div>
            <label className="text-xs text-ghost-text-dim mb-1 block">
              Temperature: {temperature.toFixed(2)}
            </label>
            <input
              type="range"
              min={0}
              max={1.5}
              step={0.05}
              value={temperature}
              onChange={(e) => setTemperature(Number(e.target.value))}
              onMouseUp={() =>
                onSave({ ...settings, chat_temperature: temperature })
              }
              className="w-full accent-ghost-accent"
            />
            <div className="flex justify-between text-[10px] text-ghost-text-dim/30">
              <span>Precise (0)</span>
              <span>Creative (1.5)</span>
            </div>
          </div>
        </div>
      </Section>

      {/* Updates — desktop only */}
      {isDesktop && (
        <Section title="Updates" icon={<Download className="w-4 h-4" />}>
          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm text-ghost-text-dim">Current version</p>
                <p className="text-xs text-ghost-text-dim/50 mt-0.5">v{__APP_VERSION__}</p>
              </div>
              <CheckUpdateButton checking={updater.checking} onCheck={updater.checkForUpdate} />
            </div>
            {updater.available && (
              <div className="p-3 bg-purple-500/10 border border-purple-500/20 rounded-lg">
                <p className="text-sm text-purple-300">
                  Ghost {updater.version} is available!
                </p>
                {updater.releaseNotes && (
                  <p className="text-xs text-white/50 mt-1 line-clamp-3">{updater.releaseNotes}</p>
                )}
                <button
                  type="button"
                  onClick={updater.installUpdate}
                  disabled={updater.downloading}
                  className="mt-2 px-4 py-1.5 bg-purple-600 hover:bg-purple-500 text-white text-xs font-medium rounded-lg transition-colors disabled:opacity-50"
                >
                  {updater.downloading ? `Downloading (${updater.progress}%)…` : "Download & Install"}
                </button>
              </div>
            )}
            {updater.error && (
              <p className="text-xs text-red-400">{updater.error}</p>
            )}
          </div>
        </Section>
      )}
    </div>
  );
}

// ─── Models Tab ────────────────────────────────────────────

function ModelsTab({
  models,
  hardware,
  onModelSwitch,
}: {
  models: ModelInfo[];
  hardware: HardwareInfo | null;
  settings: SettingsType;
  onModelSwitch: (id: string) => void;
  onModelDownload: () => void;
}) {
  const [switching, setSwitching] = useState<string | null>(null);

  const handleSwitch = async (modelId: string) => {
    setSwitching(modelId);
    onModelSwitch(modelId);
    // Reset after a delay (actual loading happens async)
    setTimeout(() => setSwitching(null), 3000);
  };

  return (
    <div className="space-y-4">
      <p className="text-xs text-ghost-text-dim/60">
        Ghost automatically selects the best model for your hardware.
        {hardware && (
          <span className="text-ghost-text-dim/40">
            {" "}Available RAM: {(hardware.available_ram_mb / 1024).toFixed(1)} GB
          </span>
        )}
      </p>

      <div className="space-y-3">
        {models.map((model) => (
          <div
            key={model.id}
            className={`p-4 rounded-xl border transition-all ${
              model.active
                ? "border-ghost-accent/50 bg-ghost-accent/5"
                : model.fits_hardware
                ? "border-ghost-border bg-ghost-bg hover:border-ghost-border hover:bg-ghost-surface-hover"
                : "border-ghost-border/50 bg-ghost-bg/50 opacity-60"
            }`}
          >
            <div className="flex items-start justify-between gap-3">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-sm font-medium text-ghost-text">
                    {model.name}
                  </span>
                  {model.active && (
                    <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-ghost-accent/20 text-ghost-accent">
                      Active
                    </span>
                  )}
                  {model.recommended && !model.active && (
                    <span className="flex items-center gap-0.5 px-1.5 py-0.5 rounded text-[10px] font-medium bg-ghost-warning/20 text-ghost-warning">
                      <Star className="w-2.5 h-2.5" />
                      Recommended
                    </span>
                  )}
                  {model.downloaded && !model.active && (
                    <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-ghost-success/20 text-ghost-success">
                      Downloaded
                    </span>
                  )}
                </div>
                <p className="text-xs text-ghost-text-dim/60 mb-2">
                  {model.description}
                </p>
                <div className="flex items-center gap-3 text-[10px] text-ghost-text-dim/40">
                  <span>{model.parameters}</span>
                  <span>·</span>
                  <span>{model.size_mb >= 1024 ? `${(model.size_mb / 1024).toFixed(1)} GB` : `${model.size_mb} MB`}</span>
                  <span>·</span>
                  <span>Min {(model.min_ram_mb / 1024).toFixed(0)} GB RAM</span>
                  <span>·</span>
                  <span>Quality: {"★".repeat(model.quality_tier)}{"☆".repeat(5 - model.quality_tier)}</span>
                </div>
              </div>

              <div className="shrink-0">
                {model.active ? (
                  <span className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-medium bg-ghost-accent text-white">
                    <Check className="w-3 h-3" />
                    In Use
                  </span>
                ) : model.downloaded ? (
                  <button
                    onClick={() => handleSwitch(model.id)}
                    disabled={switching === model.id || !model.fits_hardware}
                    className="px-3 py-1.5 rounded-lg text-xs font-medium bg-ghost-surface-hover text-ghost-text border border-ghost-border hover:bg-ghost-border disabled:opacity-40 transition-all"
                  >
                    {switching === model.id ? (
                      <Loader2 className="w-3 h-3 animate-spin" />
                    ) : (
                      "Switch"
                    )}
                  </button>
                ) : model.fits_hardware ? (
                  <button
                    onClick={() => handleSwitch(model.id)}
                    disabled={switching === model.id}
                    className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-medium bg-ghost-accent/20 text-ghost-accent hover:bg-ghost-accent/30 disabled:opacity-40 transition-all"
                  >
                    {switching === model.id ? (
                      <Loader2 className="w-3 h-3 animate-spin" />
                    ) : (
                      <>
                        <Download className="w-3 h-3" />
                        Download
                      </>
                    )}
                  </button>
                ) : (
                  <span className="px-3 py-1.5 rounded-lg text-[10px] text-ghost-text-dim/30">
                    Needs more RAM
                  </span>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ─── Directories Tab ───────────────────────────────────────

function DirectoriesTab({
  settings,
  onSave,
}: {
  settings: SettingsType;
  onSave: (s: SettingsType) => void;
}) {
  const [directories, setDirectories] = useState<string[]>(
    settings.watched_directories
  );
  const [showBrowser, setShowBrowser] = useState(false);
  const [browsePath, setBrowsePath] = useState<string | null>(null);
  const [entries, setEntries] = useState<FsEntry[]>([]);
  const [roots, setRoots] = useState<FsEntry[]>([]);
  const [browserLoading, setBrowserLoading] = useState(false);

  // Load root directories
  useEffect(() => {
    getRootDirectories()
      .then(setRoots)
      .catch(() => {});
  }, []);

  const openBrowser = useCallback(async (path?: string) => {
    setShowBrowser(true);
    setBrowserLoading(true);
    try {
      if (path) {
        setBrowsePath(path);
        const items = await listDirectory(path);
        setEntries(items.filter((e) => e.is_dir));
      } else {
        setBrowsePath(null);
        const rootItems = await getRootDirectories();
        setRoots(rootItems);
        setEntries([]);
      }
    } catch (e) {
      console.error(e);
    } finally {
      setBrowserLoading(false);
    }
  }, []);

  const navigateTo = useCallback(async (path: string) => {
    setBrowserLoading(true);
    try {
      setBrowsePath(path);
      const items = await listDirectory(path);
      setEntries(items.filter((e) => e.is_dir));
    } catch (e) {
      console.error(e);
    } finally {
      setBrowserLoading(false);
    }
  }, []);

  const navigateUp = useCallback(() => {
    if (!browsePath) return;
    const parent = browsePath.replace(/[/\\][^/\\]+$/, "") || "/";
    if (parent === browsePath) {
      setBrowsePath(null);
      setEntries([]);
      return;
    }
    navigateTo(parent);
  }, [browsePath, navigateTo]);

  const addDirectory = useCallback(
    (path: string) => {
      if (!directories.includes(path)) {
        const updated = [...directories, path];
        setDirectories(updated);
        onSave({ ...settings, watched_directories: updated });
        addWatchDirectory(path).catch(() => {});
      }
    },
    [directories, settings, onSave]
  );

  const removeDirectory = useCallback(
    (path: string) => {
      const updated = directories.filter((d) => d !== path);
      setDirectories(updated);
      onSave({ ...settings, watched_directories: updated });
      removeWatchDirectory(path).catch(() => {});
    },
    [directories, settings, onSave]
  );

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <p className="text-xs text-ghost-text-dim/60">
          Ghost indexes these directories to enable search across your files.
        </p>
        <button
          onClick={() => openBrowser()}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-ghost-accent/20 text-ghost-accent hover:bg-ghost-accent/30 transition-all"
        >
          <Plus className="w-3 h-3" />
          Add Folder
        </button>
      </div>

      {/* Current directories */}
      {directories.length === 0 ? (
        <div className="text-center py-8 text-ghost-text-dim/40 text-sm">
          No directories configured. Ghost will auto-discover on first launch.
        </div>
      ) : (
        <div className="space-y-2">
          {directories.map((dir) => (
            <div
              key={dir}
              className="flex items-center justify-between px-4 py-3 bg-ghost-bg rounded-xl border border-ghost-border group"
            >
              <div className="flex items-center gap-2 min-w-0">
                <FolderOpen className="w-4 h-4 text-ghost-accent shrink-0" />
                <span className="text-sm text-ghost-text truncate">{dir}</span>
              </div>
              <button
                onClick={() => removeDirectory(dir)}
                className="p-1.5 rounded-lg text-ghost-text-dim/30 hover:text-ghost-danger hover:bg-ghost-danger/10 opacity-0 group-hover:opacity-100 transition-all shrink-0"
                title="Remove directory"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          ))}
        </div>
      )}

      {/* File Browser Overlay */}
      {showBrowser && (
        <div className="border border-ghost-border rounded-xl bg-ghost-bg overflow-hidden">
          {/* Browser header */}
          <div className="flex items-center gap-2 px-4 py-3 border-b border-ghost-border bg-ghost-surface">
            {browsePath ? (
              <>
                <button
                  onClick={navigateUp}
                  className="p-1 rounded hover:bg-ghost-surface-hover transition-colors"
                >
                  <ArrowLeft className="w-4 h-4 text-ghost-text-dim" />
                </button>
                <span className="text-xs text-ghost-text-dim truncate flex-1">
                  {browsePath}
                </span>
                <button
                  onClick={() => addDirectory(browsePath)}
                  disabled={directories.includes(browsePath)}
                  className="px-3 py-1 rounded-lg text-xs font-medium bg-ghost-accent text-white hover:bg-ghost-accent-dim disabled:opacity-40 transition-all"
                >
                  {directories.includes(browsePath) ? (
                    <span className="flex items-center gap-1">
                      <Check className="w-3 h-3" /> Added
                    </span>
                  ) : (
                    "Add This Folder"
                  )}
                </button>
              </>
            ) : (
              <span className="text-xs text-ghost-text-dim">
                Select a folder to index
              </span>
            )}
            <button
              onClick={() => setShowBrowser(false)}
              className="p-1 rounded hover:bg-ghost-surface-hover transition-colors ml-auto"
            >
              <X className="w-4 h-4 text-ghost-text-dim" />
            </button>
          </div>

          {/* Browser content */}
          <div className="max-h-60 overflow-y-auto">
            {browserLoading ? (
              <div className="flex items-center justify-center py-8">
                <Loader2 className="w-4 h-4 animate-spin text-ghost-accent" />
              </div>
            ) : !browsePath ? (
              /* Root directories / quick access */
              <div className="p-2">
                {roots.map((root) => (
                  <button
                    key={root.path}
                    onClick={() => navigateTo(root.path)}
                    className="flex items-center gap-3 w-full px-3 py-2.5 rounded-lg text-left hover:bg-ghost-surface-hover transition-colors"
                  >
                    <RootIcon name={root.name} />
                    <div className="min-w-0">
                      <span className="text-sm text-ghost-text block">
                        {root.name}
                      </span>
                      <span className="text-[10px] text-ghost-text-dim/40 truncate block">
                        {root.path}
                      </span>
                    </div>
                    <ChevronRight className="w-4 h-4 text-ghost-text-dim/30 ml-auto shrink-0" />
                  </button>
                ))}
              </div>
            ) : entries.length === 0 ? (
              <div className="text-center py-8 text-ghost-text-dim/40 text-xs">
                No subdirectories
              </div>
            ) : (
              <div className="p-2">
                {entries.map((entry) => (
                  <button
                    key={entry.path}
                    onClick={() => navigateTo(entry.path)}
                    className="flex items-center gap-3 w-full px-3 py-2 rounded-lg text-left hover:bg-ghost-surface-hover transition-colors group"
                  >
                    <FolderOpen
                      className={`w-4 h-4 shrink-0 ${
                        entry.is_cloud_placeholder
                          ? "text-blue-400/60"
                          : "text-ghost-accent/60"
                      }`}
                    />
                    <span className="text-sm text-ghost-text truncate flex-1">
                      {entry.name}
                    </span>
                    {entry.is_cloud_placeholder && (
                      <span title="Cloud file (not downloaded)"><Cloud className="w-3 h-3 text-blue-400/40" /></span>
                    )}
                    <ChevronRight className="w-3.5 h-3.5 text-ghost-text-dim/20 group-hover:text-ghost-text-dim/40 shrink-0" />
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

// ─── MCP Tab ───────────────────────────────────────────────

function McpTab({ onError }: { onError: (msg: string) => void }) {
  const [serverStatus, setServerStatus] = useState<McpServerStatus | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getMcpServerStatus()
      .then(setServerStatus)
      .catch((e) => onError(String(e)))
      .finally(() => setLoading(false));
  }, [onError]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-5 h-5 animate-spin text-ghost-text-dim" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Ghost MCP Server Status */}
      <Section title="Ghost MCP Server" icon={<Plug className="w-4 h-4" />}>
        <div className="px-4 py-3 bg-ghost-bg rounded-xl border border-ghost-border">
          {serverStatus ? (
            <div className="flex items-center justify-between">
              <div>
                <div className="flex items-center gap-2 mb-1">
                  <span
                    className={`w-2 h-2 rounded-full ${
                      serverStatus.enabled ? "bg-green-400" : "bg-ghost-text-dim/30"
                    }`}
                  />
                  <span className="text-sm font-medium text-ghost-text">
                    {serverStatus.enabled ? "Running" : "Disabled"}
                  </span>
                </div>
                {serverStatus.enabled && (
                  <p className="text-xs text-ghost-text-dim ml-4">
                    {serverStatus.url}
                  </p>
                )}
              </div>
              <p className="text-xs text-ghost-text-dim/60">
                Connect Claude Desktop, Cursor, or VS Code Copilot
              </p>
            </div>
          ) : (
            <p className="text-sm text-ghost-text-dim">Unable to get server status</p>
          )}
        </div>
      </Section>

      {/* MCP App Store */}
      <McpAppStore onError={onError} />
    </div>
  );
}

// ─── Shared Components ─────────────────────────────────────

function Section({
  title,
  icon,
  children,
}: {
  title: string;
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <section>
      <h3 className="flex items-center gap-2 text-sm font-medium text-ghost-text mb-3">
        {icon}
        {title}
      </h3>
      {children}
    </section>
  );
}

function InfoCard({ label, value }: { label: string; value: string }) {
  return (
    <div className="px-3 py-2.5 bg-ghost-bg rounded-lg border border-ghost-border">
      <span className="text-[10px] text-ghost-text-dim/40 block mb-0.5">
        {label}
      </span>
      <span className="text-sm font-medium text-ghost-text">{value}</span>
    </div>
  );
}

function RootIcon({ name }: { name: string }) {
  const lower = name.toLowerCase();
  if (lower === "home") return <Home className="w-4 h-4 text-ghost-accent shrink-0" />;
  if (lower.includes("onedrive") || lower.includes("cloud"))
    return <Cloud className="w-4 h-4 text-blue-400 shrink-0" />;
  if (lower === "/" || lower.includes("drive"))
    return <HardDrive className="w-4 h-4 text-ghost-text-dim shrink-0" />;
  return <FolderOpen className="w-4 h-4 text-ghost-accent/60 shrink-0" />;
}
