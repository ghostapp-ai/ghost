/**
 * A2UIRenderer — React renderer for Google A2UI v0.9 generative UI.
 *
 * Maps A2UI component definitions (adjacency list model from JSON)
 * to React/Tailwind native components. This is one of the first
 * React implementations of the A2UI standard.
 *
 * The renderer:
 * 1. Receives an A2uiSurfaceState (components + data model)
 * 2. Resolves the adjacency list into a component tree
 * 3. Renders each component type with appropriate Tailwind styling
 * 4. Handles data binding (read from data model, write via callbacks)
 * 5. Dispatches actions back to the agent via AG-UI events
 *
 * Supported component catalog:
 *   Layout: Row, Column, Card, Divider, Tabs, Modal, List
 *   Content: Text, Image, Icon, Video
 *   Input: Button, TextField, CheckBox, ChoicePicker, Slider, DateTimeInput
 */

import { useCallback } from "react";
import type {
  A2uiSurfaceState,
  A2uiComponent,
  A2uiAction,
} from "../lib/types";

// ---------------------------------------------------------------------------
// Props
// ---------------------------------------------------------------------------

interface A2UIRendererProps {
  /** The surface state containing components + data model. */
  surface: A2uiSurfaceState;
  /** Callback when a component action is triggered (e.g., button click). */
  onAction?: (surfaceId: string, componentId: string, action: A2uiAction) => void;
  /** Callback when an input value changes (two-way binding). */
  onDataChange?: (surfaceId: string, path: string, value: unknown) => void;
}

// ---------------------------------------------------------------------------
// Data Model Resolution
// ---------------------------------------------------------------------------

/** Resolve a JSON Pointer path against the data model. */
function resolvePointer(dataModel: Record<string, unknown>, pointer: string): unknown {
  if (!pointer || pointer === "/") return dataModel;
  const parts = pointer.replace(/^\//, "").split("/");
  let current: unknown = dataModel;
  for (const part of parts) {
    if (current == null || typeof current !== "object") return undefined;
    current = (current as Record<string, unknown>)[part];
  }
  return current;
}

/** Resolve a dynamic string (literal, path binding, or function call). */
function resolveDynamicString(
  value: string | { path: string } | { call: string; args: unknown } | undefined,
  dataModel: Record<string, unknown>
): string {
  if (value === undefined || value === null) return "";
  if (typeof value === "string") return value;
  if ("path" in value) {
    const resolved = resolvePointer(dataModel, value.path);
    return resolved != null ? String(resolved) : "";
  }
  if ("call" in value) {
    // Built-in formatString function support
    if (value.call === "formatString" && typeof value.args === "object" && value.args !== null) {
      const args = value.args as Record<string, unknown>;
      let template = String(args.template ?? "");
      const params = (args.args ?? []) as unknown[];
      params.forEach((arg, i) => {
        let resolved = arg;
        if (typeof arg === "object" && arg !== null && "path" in (arg as Record<string, unknown>)) {
          resolved = resolvePointer(dataModel, (arg as { path: string }).path);
        }
        template = template.replace(`{${i}}`, String(resolved ?? ""));
      });
      return template;
    }
    return `[fn:${value.call}]`;
  }
  return "";
}

/** Resolve a dynamic value (for input bindings). */
function resolveDynamicValue(
  value: string | number | boolean | { path: string } | undefined,
  dataModel: Record<string, unknown>
): unknown {
  if (value === undefined || value === null) return undefined;
  if (typeof value === "object" && "path" in value) {
    return resolvePointer(dataModel, value.path);
  }
  return value;
}

/** Get the binding path from a value definition. */
function getBindingPath(
  value: string | number | boolean | { path: string } | undefined
): string | null {
  if (value !== undefined && value !== null && typeof value === "object" && "path" in value) {
    return value.path;
  }
  return null;
}

// ---------------------------------------------------------------------------
// Component Registry
// ---------------------------------------------------------------------------

/** Render a single A2UI component by type. */
function RenderComponent({
  comp,
  surface,
  onAction,
  onDataChange,
}: {
  comp: A2uiComponent;
  surface: A2uiSurfaceState;
  onAction?: A2UIRendererProps["onAction"];
  onDataChange?: A2UIRendererProps["onDataChange"];
}) {
  const { dataModel, components: componentMap, surfaceId } = surface;
  const type = comp.component;

  /** Render a child component by ID. */
  const renderChild = (childId: string) => {
    const child = componentMap.get(childId);
    if (!child) return null;
    return (
      <RenderComponent
        key={childId}
        comp={child}
        surface={surface}
        onAction={onAction}
        onDataChange={onDataChange}
      />
    );
  };

  /** Render an array of child IDs. */
  const renderChildren = (childIds: string[]) =>
    childIds.map((id) => renderChild(id));

  /** Resolve children prop (static array or template). */
  const getChildIds = (): string[] => {
    if (!comp.children) return [];
    if (Array.isArray(comp.children)) return comp.children;
    // Template children — not yet supported in renderer
    return [];
  };

  switch (type) {
    // -----------------------------------------------------------------------
    // Content Components
    // -----------------------------------------------------------------------

    case "Text": {
      const text = resolveDynamicString(comp.text, dataModel);
      const variant = comp.variant ?? "body";
      const variantClasses: Record<string, string> = {
        h1: "text-xl font-bold text-ghost-text",
        h2: "text-lg font-semibold text-ghost-text",
        h3: "text-base font-semibold text-ghost-text",
        body: "text-sm text-ghost-text",
        caption: "text-xs text-ghost-text-dim/60",
        code: "text-xs font-mono bg-ghost-surface-hover px-1.5 py-0.5 rounded",
      };
      return (
        <span
          className={variantClasses[variant] ?? variantClasses.body}
          data-a2ui-id={comp.id}
        >
          {text}
        </span>
      );
    }

    case "Image": {
      const url = resolveDynamicString(comp.url, dataModel);
      return (
        <img
          src={url}
          alt={resolveDynamicString(comp.text, dataModel) || ""}
          className="max-w-full rounded-lg"
          data-a2ui-id={comp.id}
        />
      );
    }

    case "Icon": {
      const name = comp.name ?? "circle";
      // Render as emoji or text fallback — full icon library integration is Phase 2
      return (
        <span
          className="inline-flex items-center justify-center w-5 h-5 text-ghost-accent text-xs"
          data-a2ui-id={comp.id}
          role="img"
          aria-label={name}
        >
          {getIconFallback(name)}
        </span>
      );
    }

    case "Video": {
      const url = resolveDynamicString(comp.url, dataModel);
      return (
        <video
          src={url}
          controls
          className="max-w-full rounded-lg"
          data-a2ui-id={comp.id}
        />
      );
    }

    // -----------------------------------------------------------------------
    // Layout Components
    // -----------------------------------------------------------------------

    case "Row": {
      const align = comp.align ?? "center";
      const justify = comp.justify ?? "start";
      return (
        <div
          className={`flex flex-row gap-2 items-${align} justify-${justify} flex-wrap`}
          data-a2ui-id={comp.id}
        >
          {renderChildren(getChildIds())}
        </div>
      );
    }

    case "Column": {
      const align = comp.align ?? "stretch";
      const justify = comp.justify ?? "start";
      return (
        <div
          className={`flex flex-col gap-2 items-${align} justify-${justify}`}
          data-a2ui-id={comp.id}
        >
          {renderChildren(getChildIds())}
        </div>
      );
    }

    case "Card": {
      return (
        <div
          className="rounded-xl border border-ghost-border/50 bg-ghost-surface p-3 shadow-sm"
          data-a2ui-id={comp.id}
        >
          {comp.child && renderChild(comp.child)}
          {renderChildren(getChildIds())}
        </div>
      );
    }

    case "Divider": {
      const axis = comp.axis ?? "horizontal";
      return axis === "horizontal" ? (
        <hr
          className="border-ghost-border/30 my-1"
          data-a2ui-id={comp.id}
        />
      ) : (
        <div
          className="w-px bg-ghost-border/30 self-stretch mx-1"
          data-a2ui-id={comp.id}
        />
      );
    }

    case "List": {
      return (
        <div
          className="flex flex-col gap-1"
          data-a2ui-id={comp.id}
        >
          {renderChildren(getChildIds())}
        </div>
      );
    }

    case "Tabs": {
      // Simplified tabs — render all children vertically for now
      return (
        <div
          className="flex flex-col gap-2"
          data-a2ui-id={comp.id}
        >
          {renderChildren(getChildIds())}
        </div>
      );
    }

    case "Modal": {
      return (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
          data-a2ui-id={comp.id}
        >
          <div className="bg-ghost-bg rounded-2xl border border-ghost-border/50 p-4 max-w-md w-full mx-4 shadow-xl">
            {comp.child && renderChild(comp.child)}
          </div>
        </div>
      );
    }

    // -----------------------------------------------------------------------
    // Input Components
    // -----------------------------------------------------------------------

    case "Button": {
      const variant = comp.variant ?? "primary";
      const baseClasses =
        "inline-flex items-center justify-center gap-1.5 px-3 py-1.5 rounded-lg text-sm font-medium transition-all cursor-pointer";
      const variantClasses: Record<string, string> = {
        primary: "bg-ghost-accent text-white hover:bg-ghost-accent/90 active:scale-[0.98]",
        secondary: "bg-ghost-surface-hover text-ghost-text hover:bg-ghost-surface-hover/80",
        borderless: "text-ghost-accent hover:bg-ghost-accent/10",
        danger: "bg-ghost-danger text-white hover:bg-ghost-danger/90",
      };
      return (
        <button
          type="button"
          className={`${baseClasses} ${variantClasses[variant] ?? variantClasses.primary}`}
          onClick={() => {
            if (comp.action && onAction) {
              onAction(surfaceId, comp.id, comp.action);
            }
          }}
          data-a2ui-id={comp.id}
        >
          {comp.child && renderChild(comp.child)}
        </button>
      );
    }

    case "TextField": {
      const label = resolveDynamicString(comp.label, dataModel);
      const currentValue = String(resolveDynamicValue(comp.value, dataModel) ?? "");
      const bindingPath = getBindingPath(comp.value);
      const variant = comp.variant ?? "shortText";

      const handleChange = useCallback(
        (newValue: string) => {
          if (bindingPath && onDataChange) {
            onDataChange(surfaceId, bindingPath, newValue);
          }
        },
        [bindingPath, onDataChange, surfaceId]
      );

      return (
        <div className="flex flex-col gap-1" data-a2ui-id={comp.id}>
          {label && (
            <label className="text-xs font-medium text-ghost-text-dim/70">
              {label}
            </label>
          )}
          {variant === "longText" ? (
            <textarea
              value={currentValue}
              onChange={(e) => handleChange(e.target.value)}
              className="w-full px-2.5 py-1.5 rounded-lg border border-ghost-border/50 bg-ghost-surface text-sm text-ghost-text placeholder-ghost-text-dim/30 focus:outline-none focus:border-ghost-accent/50 resize-y min-h-[60px]"
              rows={3}
            />
          ) : (
            <input
              type="text"
              value={currentValue}
              onChange={(e) => handleChange(e.target.value)}
              className="w-full px-2.5 py-1.5 rounded-lg border border-ghost-border/50 bg-ghost-surface text-sm text-ghost-text placeholder-ghost-text-dim/30 focus:outline-none focus:border-ghost-accent/50"
            />
          )}
        </div>
      );
    }

    case "CheckBox": {
      const label = resolveDynamicString(comp.label, dataModel);
      const checked = resolveDynamicValue(comp.value, dataModel) === true;
      const bindingPath = getBindingPath(comp.value);

      return (
        <label
          className="inline-flex items-center gap-2 cursor-pointer"
          data-a2ui-id={comp.id}
        >
          <input
            type="checkbox"
            checked={checked}
            onChange={(e) => {
              if (bindingPath && onDataChange) {
                onDataChange(surfaceId, bindingPath, e.target.checked);
              }
            }}
            className="w-4 h-4 rounded border-ghost-border/50 text-ghost-accent focus:ring-ghost-accent/30"
          />
          <span className="text-sm text-ghost-text">{label}</span>
        </label>
      );
    }

    case "ChoicePicker": {
      const options = comp.options ?? [];
      const currentValue = String(resolveDynamicValue(comp.value, dataModel) ?? "");
      const bindingPath = getBindingPath(comp.value);
      const variant = comp.variant ?? "dropdown";

      if (variant === "radio" || variant === "chip") {
        return (
          <div
            className="flex flex-wrap gap-2"
            data-a2ui-id={comp.id}
          >
            {options.map((opt) => (
              <button
                key={opt.value}
                type="button"
                className={`px-3 py-1 rounded-lg text-xs font-medium transition-all ${
                  currentValue === opt.value
                    ? "bg-ghost-accent text-white"
                    : "bg-ghost-surface-hover text-ghost-text hover:bg-ghost-surface-hover/80"
                }`}
                onClick={() => {
                  if (bindingPath && onDataChange) {
                    onDataChange(surfaceId, bindingPath, opt.value);
                  }
                }}
              >
                {opt.label}
              </button>
            ))}
          </div>
        );
      }

      // Default: dropdown/select
      return (
        <select
          value={currentValue}
          onChange={(e) => {
            if (bindingPath && onDataChange) {
              onDataChange(surfaceId, bindingPath, e.target.value);
            }
          }}
          className="px-2.5 py-1.5 rounded-lg border border-ghost-border/50 bg-ghost-surface text-sm text-ghost-text focus:outline-none focus:border-ghost-accent/50"
          data-a2ui-id={comp.id}
        >
          {options.map((opt) => (
            <option key={opt.value} value={opt.value}>
              {opt.label}
            </option>
          ))}
        </select>
      );
    }

    case "Slider": {
      const currentValue = Number(resolveDynamicValue(comp.value, dataModel) ?? comp.min ?? 0);
      const bindingPath = getBindingPath(comp.value);
      const label = resolveDynamicString(comp.label, dataModel);

      return (
        <div className="flex flex-col gap-1" data-a2ui-id={comp.id}>
          {label && (
            <div className="flex items-center justify-between">
              <label className="text-xs font-medium text-ghost-text-dim/70">
                {label}
              </label>
              <span className="text-xs font-mono text-ghost-text-dim/50">
                {currentValue}
              </span>
            </div>
          )}
          <input
            type="range"
            min={comp.min ?? 0}
            max={comp.max ?? 100}
            step={comp.step ?? 1}
            value={currentValue}
            onChange={(e) => {
              if (bindingPath && onDataChange) {
                onDataChange(surfaceId, bindingPath, Number(e.target.value));
              }
            }}
            className="w-full accent-ghost-accent"
          />
        </div>
      );
    }

    case "DateTimeInput": {
      const label = resolveDynamicString(comp.label, dataModel);
      const currentValue = String(resolveDynamicValue(comp.value, dataModel) ?? "");
      const bindingPath = getBindingPath(comp.value);

      return (
        <div className="flex flex-col gap-1" data-a2ui-id={comp.id}>
          {label && (
            <label className="text-xs font-medium text-ghost-text-dim/70">
              {label}
            </label>
          )}
          <input
            type="datetime-local"
            value={currentValue}
            onChange={(e) => {
              if (bindingPath && onDataChange) {
                onDataChange(surfaceId, bindingPath, e.target.value);
              }
            }}
            className="px-2.5 py-1.5 rounded-lg border border-ghost-border/50 bg-ghost-surface text-sm text-ghost-text focus:outline-none focus:border-ghost-accent/50"
          />
        </div>
      );
    }

    // -----------------------------------------------------------------------
    // Fallback
    // -----------------------------------------------------------------------
    default:
      return (
        <div
          className="text-xs text-ghost-text-dim/40 italic px-1"
          data-a2ui-id={comp.id}
        >
          [A2UI: {type} #{comp.id}]
        </div>
      );
  }
}

// ---------------------------------------------------------------------------
// Icon Fallback Map
// ---------------------------------------------------------------------------

/** Simple emoji/text fallback for common icon names. */
function getIconFallback(name: string): string {
  const icons: Record<string, string> = {
    search: "🔍",
    check: "✓",
    close: "✕",
    add: "+",
    remove: "−",
    star: "★",
    heart: "♥",
    info: "ℹ",
    warning: "⚠",
    error: "✕",
    settings: "⚙",
    home: "🏠",
    file: "📄",
    folder: "📁",
    download: "⬇",
    upload: "⬆",
    edit: "✏",
    delete: "🗑",
    copy: "📋",
    send: "➤",
    circle: "●",
  };
  return icons[name.toLowerCase()] ?? "●";
}

// ---------------------------------------------------------------------------
// Surface Renderer
// ---------------------------------------------------------------------------

/**
 * Render a complete A2UI surface.
 *
 * Resolves the adjacency list model into a tree by finding root components
 * (those not referenced as children by any other component) and renders
 * them in order.
 */
export function A2UISurface({
  surface,
  onAction,
  onDataChange,
}: A2UIRendererProps) {
  const theme = surface.theme;

  return (
    <div
      className="a2ui-surface flex flex-col gap-2"
      data-a2ui-surface={surface.surfaceId}
      style={theme?.primaryColor ? { "--a2ui-primary": theme.primaryColor } as React.CSSProperties : undefined}
    >
      {surface.rootIds.map((rootId) => {
        const comp = surface.components.get(rootId);
        if (!comp) return null;
        return (
          <RenderComponent
            key={rootId}
            comp={comp}
            surface={surface}
            onAction={onAction}
            onDataChange={onDataChange}
          />
        );
      })}
    </div>
  );
}

/**
 * Render all A2UI surfaces from a run state.
 *
 * This component renders all active surfaces received during an AG-UI run.
 */
export function A2UIRenderer({
  surfaces,
  onAction,
  onDataChange,
}: {
  surfaces: Map<string, A2uiSurfaceState>;
  onAction?: A2UIRendererProps["onAction"];
  onDataChange?: A2UIRendererProps["onDataChange"];
}) {
  if (surfaces.size === 0) return null;

  return (
    <div className="a2ui-renderer flex flex-col gap-3">
      {Array.from(surfaces.values()).map((surface) => (
        <A2UISurface
          key={surface.surfaceId}
          surface={surface}
          onAction={onAction}
          onDataChange={onDataChange}
        />
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Utility: Compute Root Component IDs
// ---------------------------------------------------------------------------

/**
 * Given a flat list of A2UI components, compute which are roots
 * (not referenced as `child` or in `children` by any other component).
 */
export function computeRootIds(components: A2uiComponent[]): string[] {
  const referencedIds = new Set<string>();

  for (const comp of components) {
    if (comp.child) referencedIds.add(comp.child);
    if (comp.children && Array.isArray(comp.children)) {
      for (const childId of comp.children) {
        referencedIds.add(childId);
      }
    }
  }

  // Roots = all IDs minus referenced IDs, preserving declaration order
  return components
    .filter((c) => !referencedIds.has(c.id))
    .map((c) => c.id);
}
