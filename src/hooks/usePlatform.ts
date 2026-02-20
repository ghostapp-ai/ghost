import { useState, useEffect, useMemo } from "react";
import { getPlatformInfo, type PlatformInfo } from "../lib/tauri";

/** Default platform info (assumes desktop for SSR/initial render). */
const DEFAULT_PLATFORM: PlatformInfo = {
  platform: "unknown",
  is_desktop: true,
  is_mobile: false,
  has_file_watcher: true,
  has_system_tray: true,
  has_global_shortcuts: true,
  has_stdio_mcp: true,
};

/**
 * Hook to detect the current platform and available capabilities.
 *
 * Returns platform info that the UI can use to:
 * - Show/hide desktop-only features (tray, shortcuts, file watcher)
 * - Adapt layout for mobile (larger touch targets, no keyboard hints)
 * - Enable/disable MCP stdio transport on mobile
 *
 * @example
 * ```tsx
 * const { platform, isMobile, isDesktop } = usePlatform();
 * if (isMobile) return <MobileLayout />;
 * ```
 */
export function usePlatform() {
  const [info, setInfo] = useState<PlatformInfo>(DEFAULT_PLATFORM);

  useEffect(() => {
    getPlatformInfo()
      .then(setInfo)
      .catch(() => {
        // Fallback: detect from user agent on web/PWA
        const ua = navigator.userAgent.toLowerCase();
        const isMobile =
          /android|iphone|ipad|ipod|mobile/i.test(ua) ||
          ("ontouchstart" in window && navigator.maxTouchPoints > 0);

        setInfo({
          platform: isMobile ? "android" : "unknown",
          is_desktop: !isMobile,
          is_mobile: isMobile,
          has_file_watcher: !isMobile,
          has_system_tray: !isMobile,
          has_global_shortcuts: !isMobile,
          has_stdio_mcp: !isMobile,
        });
      });
  }, []);

  return useMemo(
    () => ({
      ...info,
      /** Shorthand: is this a mobile platform? */
      isMobile: info.is_mobile,
      /** Shorthand: is this a desktop platform? */
      isDesktop: info.is_desktop,
      /** Is this iOS specifically? */
      isIos: info.platform === "ios",
      /** Is this Android specifically? */
      isAndroid: info.platform === "android",
      /** Is this macOS specifically? */
      isMacos: info.platform === "macos",
      /** Shortcut key label: Cmd on macOS, Ctrl everywhere else */
      modKey: info.platform === "macos" ? "⌘" : "Ctrl",
      /** Activation shortcut label */
      activationShortcut:
        info.platform === "macos" ? "⌘+Space" : "Ctrl+Space",
    }),
    [info],
  );
}
