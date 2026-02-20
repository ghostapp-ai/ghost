import {
  FileText,
  FileSpreadsheet,
  FileCode,
  File,
  FileImage,
  Braces,
} from "lucide-react";
import type { SearchResult } from "../lib/types";

interface ResultItemProps {
  result: SearchResult;
  isSelected: boolean;
  onSelect: () => void;
  onOpen: () => void;
  /** Whether the app is running on a mobile device */
  isMobile?: boolean;
}

const EXT_ICONS: Record<string, typeof FileText> = {
  pdf: FileText,
  docx: FileText,
  txt: FileText,
  md: FileCode,
  markdown: FileCode,
  json: Braces,
  yaml: Braces,
  yml: Braces,
  toml: Braces,
  xml: FileCode,
  html: FileCode,
  htm: FileCode,
  csv: FileSpreadsheet,
  xlsx: FileSpreadsheet,
  xls: FileSpreadsheet,
  ods: FileSpreadsheet,
  png: FileImage,
  jpg: FileImage,
  jpeg: FileImage,
};

const SOURCE_COLORS: Record<string, string> = {
  hybrid: "bg-ghost-accent/20 text-ghost-accent",
  fts: "bg-emerald-500/20 text-emerald-400",
  vector: "bg-amber-500/20 text-amber-400",
};

function getIcon(extension: string | null) {
  if (!extension) return File;
  return EXT_ICONS[extension.toLowerCase()] || File;
}

function formatPath(path: string): string {
  // Show relative-ish path, abbreviating home dir
  const home = path.replace(/\\/g, "/");
  const parts = home.split("/");
  if (parts.length > 4) {
    return `.../${parts.slice(-3).join("/")}`;
  }
  return home;
}

export function ResultItem({ result, isSelected, onSelect, onOpen, isMobile = false }: ResultItemProps) {
  const Icon = getIcon(result.extension);

  return (
    <button
      onClick={isMobile ? onOpen : onSelect}
      onDoubleClick={isMobile ? undefined : onOpen}
      className={`
        w-full text-left px-4 ${isMobile ? "py-3.5" : "py-3"} rounded-xl transition-all duration-150
        border border-transparent
        ${
          isSelected
            ? "bg-ghost-accent/10 border-ghost-accent/30"
            : "hover:bg-ghost-surface-hover active:bg-ghost-surface-hover"
        }
      `}
      aria-selected={isSelected}
      role="option"
    >
      <div className="flex items-start gap-3">
        <div
          className={`mt-0.5 p-1.5 rounded-lg shrink-0 ${
            isSelected
              ? "bg-ghost-accent/20 text-ghost-accent"
              : "bg-ghost-surface-hover text-ghost-text-dim"
          }`}
        >
          <Icon className="w-4 h-4" />
        </div>

        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            <span className="text-sm font-medium text-ghost-text truncate">
              {result.filename}
            </span>
            <span
              className={`text-[10px] font-medium px-1.5 py-0.5 rounded-full uppercase tracking-wider shrink-0 ${
                SOURCE_COLORS[result.source] || SOURCE_COLORS.fts
              }`}
            >
              {result.source}
            </span>
          </div>

          <p className="text-xs text-ghost-text-dim/80 line-clamp-2 leading-relaxed">
            {result.snippet}
          </p>

          <div className="flex items-center gap-2 mt-1.5">
            <span className="text-[11px] text-ghost-text-dim/60 truncate">
              {formatPath(result.path)}
            </span>
            <span className="text-[11px] text-ghost-text-dim/40 tabular-nums shrink-0">
              {(result.score * 100).toFixed(1)}%
            </span>
          </div>
        </div>
      </div>
    </button>
  );
}
