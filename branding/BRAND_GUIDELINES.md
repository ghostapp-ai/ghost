# Ghost — Brand Guidelines

> Visual identity system for the Ghost desktop application.
> Last updated: February 18, 2026

---

## Brand Overview

**Ghost** is a private, local-first AI assistant for desktop. The brand identity reflects:
- **Privacy & Trust** — dark, contained, safe
- **Intelligence** — subtle glow, awareness (eyes)
- **Speed & Minimalism** — clean lines, no clutter
- **Approachability** — friendly ghost character, not intimidating

---

## Logo

### Primary Mark
The Ghost icon is a friendly ghost silhouette with distinctive glowing eyes, rendered with a purple gradient. It works as both an app icon and brand mark.

| Variant | File | Usage |
| ------- | ---- | ----- |
| Icon (transparent) | `branding/svg/ghost-icon.svg` | In-app header, overlays on dark backgrounds |
| Icon (rounded square) | `branding/svg/ghost-icon-rounded.svg` | **Primary app icon** — Windows, Linux, macOS, store listings |
| Icon (circle) | `branding/svg/ghost-icon-circle.svg` | Social media avatars, badges |
| Favicon | `branding/svg/ghost-favicon.svg` | Browser tab icon (simplified for 16px) |
| Tray icon | `branding/svg/ghost-tray-icon.svg` | System tray/menubar (monochrome-friendly) |
| Mono (white) | `branding/svg/ghost-mono-white.svg` | Watermarks, dark backgrounds, print |
| Mono (dark) | `branding/svg/ghost-mono-dark.svg` | Light backgrounds, print |

### Wordmark
| Variant | File | Usage |
| ------- | ---- | ----- |
| Dark background | `branding/svg/ghost-wordmark-dark.svg` | Website headers, dark UIs |
| Light background | `branding/svg/ghost-wordmark-light.svg` | Print, light UIs |

### Clear Space
Always maintain a minimum clear space around the logo equal to the height of the ghost's eyes. Never crowd the logo with other elements.

### Minimum Size
- **Icon**: 16x16px minimum (use favicon variant below 24px)
- **Wordmark**: 120px wide minimum

### Don'ts
- ❌ Don't rotate or skew the logo
- ❌ Don't change the gradient colors
- ❌ Don't add drop shadows beyond the built-in glow
- ❌ Don't place the colored logo on busy/colorful backgrounds
- ❌ Don't stretch or distort proportions

---

## Color Palette

### Primary Colors

| Name | Hex | RGB | Usage |
| ---- | --- | --- | ----- |
| **Ghost Accent** | `#6c5ce7` | 108, 92, 231 | Primary brand color, buttons, links, active states |
| **Ghost Accent Dim** | `#5a4bd6` | 90, 75, 214 | Hover states, secondary accents |
| **Ghost Accent Light** | `#7c6df0` | 124, 109, 240 | Gradient start, highlights |
| **Ghost Accent Glow** | `#a78bfa` | 167, 139, 250 | Highlights, subtle glows |

### Background Colors (Dark Theme)

| Name | Hex | CSS Variable | Usage |
| ---- | --- | ------------ | ----- |
| **Background** | `#0a0a0f` | `--color-ghost-bg` | Main app background |
| **Surface** | `#12121a` | `--color-ghost-surface` | Cards, panels, modals |
| **Surface Hover** | `#1a1a26` | `--color-ghost-surface-hover` | Interactive surface states |
| **Border** | `#1e1e2e` | `--color-ghost-border` | Dividers, outlines |

### Text Colors

| Name | Hex | CSS Variable | Usage |
| ---- | --- | ------------ | ----- |
| **Primary Text** | `#e4e4ef` | `--color-ghost-text` | Headings, body text |
| **Dim Text** | `#8888a0` | `--color-ghost-text-dim` | Labels, metadata, hints |

### Semantic Colors

| Name | Hex | CSS Variable | Usage |
| ---- | --- | ------------ | ----- |
| **Success** | `#00d68f` | `--color-ghost-success` | Positive states, connected |
| **Warning** | `#ffaa00` | `--color-ghost-warning` | Caution, degraded |
| **Danger** | `#ff3d71` | `--color-ghost-danger` | Errors, destructive actions |

### Gradient Definitions

```css
/* Primary brand gradient */
background: linear-gradient(135deg, #7c6df0 0%, #6c5ce7 50%, #5a4bd6 100%);

/* Subtle glow effect */
box-shadow: 0 0 24px rgba(108, 92, 231, 0.3);

/* Background radial (for hero sections) */
background: radial-gradient(ellipse at 50% 40%, #161626 0%, #0a0a14 100%);
```

---

## Typography

### Font Stack
```css
font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
```

### Scale

| Purpose | Size | Weight | Letter Spacing |
| ------- | ---- | ------ | -------------- |
| **App Title** | 14px | 600 (Semibold) | -0.01em |
| **Section Header** | 13px | 600 (Semibold) | 0 |
| **Body Text** | 13px | 400 (Regular) | 0 |
| **Small Text** | 12px | 400 (Regular) | 0 |
| **Caption/Meta** | 11px | 400 (Regular) | 0.01em |
| **Tiny** | 10px | 400 (Regular) | 0.02em |
| **Monospace** | 12px | 400 | 0 |

### Monospace Font  
```css
font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace;
```

---

## Iconography

### System Icons
Ghost uses [Lucide](https://lucide.dev) icons throughout the interface.

| Guideline | Value |
| --------- | ----- |
| Default size | 16px (w-4 h-4) |
| Small size | 12px (w-3 h-3) |
| Stroke width | 1.75px (default) |
| Color | Inherits text color |

### Custom Icons
If Lucide doesn't have what you need, create SVG icons following these rules:
- 24x24 viewBox
- 1.75px stroke, round caps and joins
- No fills (stroke-only)
- Match Lucide's visual weight

---

## Asset Inventory

### Generated Files

#### Tauri App Icons (`src-tauri/icons/`)
| File | Size | Platform |
| ---- | ---- | -------- |
| `32x32.png` | 32×32 | All |
| `128x128.png` | 128×128 | All |
| `128x128@2x.png` | 256×256 | macOS Retina |
| `icon.png` | 512×512 | Fallback |
| `icon.ico` | Multi-size | Windows |
| `icon.icns` | Multi-size | macOS |
| `StoreLogo.png` | 50×50 | Windows Store |
| `Square30x30Logo.png` | 30×30 | Windows Store |
| `Square44x44Logo.png` | 44×44 | Windows Store |
| `Square71x71Logo.png` | 71×71 | Windows Store |
| `Square89x89Logo.png` | 89×89 | Windows Store |
| `Square107x107Logo.png` | 107×107 | Windows Store |
| `Square142x142Logo.png` | 142×142 | Windows Store |
| `Square150x150Logo.png` | 150×150 | Windows Store |
| `Square284x284Logo.png` | 284×284 | Windows Store |
| `Square310x310Logo.png` | 310×310 | Windows Store |

#### Web Assets (`public/`)
| File | Size | Usage |
| ---- | ---- | ----- |
| `favicon.svg` | Scalable | Browser tab (modern browsers) |
| `favicon.ico` | 16+32+48 | Browser tab (legacy) |
| `favicon-16x16.png` | 16×16 | Browser tab |
| `favicon-32x32.png` | 32×32 | Browser tab |
| `apple-touch-icon.png` | 180×180 | iOS home screen |
| `icon-192.png` | 192×192 | Android/Chrome |
| `icon-512.png` | 512×512 | Android splash |
| `ghost-logo.svg` | Scalable | In-app display |
| `site.webmanifest` | — | PWA manifest |

#### Brand Exports (`branding/png/`)
All sizes from 16px to 1024px for icon, rounded, circle, mono, and tray variants.

#### Social Media (`branding/social/`)
| File | Size | Usage |
| ---- | ---- | ----- |
| `github-avatar.png` | 500×500 | GitHub org/profile |
| `og-card.png` | 1200×630 | Open Graph / Twitter Card |
| `og-icon.png` | 630×630 | Fallback social icon |

---

## Regenerating Assets

All PNG, ICO, and platform assets are generated from the master SVGs. To regenerate after modifying an SVG:

```bash
./branding/scripts/generate-icons.sh
```

### Requirements
- `resvg` — `cargo install resvg`
- `convert` (ImageMagick) — `sudo apt install imagemagick`
- Optional: `png2icns` for proper macOS ICNS (or use `iconutil` on macOS)

### Modifying the Brand
1. Edit the SVG files in `branding/svg/`
2. Run the generation script
3. Verify icons at all sizes look correct
4. Commit both SVGs and generated PNGs

---

## Application Theming

### CSS Variables (defined in `src/styles/globals.css`)
```css
@theme {
  --color-ghost-bg: #0a0a0f;
  --color-ghost-surface: #12121a;
  --color-ghost-surface-hover: #1a1a26;
  --color-ghost-border: #1e1e2e;
  --color-ghost-text: #e4e4ef;
  --color-ghost-text-dim: #8888a0;
  --color-ghost-accent: #6c5ce7;
  --color-ghost-accent-dim: #5a4bd6;
  --color-ghost-success: #00d68f;
  --color-ghost-warning: #ffaa00;
  --color-ghost-danger: #ff3d71;
}
```

### Component Patterns
- **Rounded corners**: `rounded-xl` (12px) for containers, `rounded-lg` (8px) for buttons
- **Border opacity**: Use `border-ghost-border/50` for subtle borders
- **Hover transitions**: `transition-all` with 150ms duration
- **Focus rings**: `ring-2 ring-ghost-accent/50 ring-offset-2 ring-offset-ghost-bg`

---

## File Structure

```
branding/
├── svg/                          # Master SVG sources (edit these)
│   ├── ghost-icon.svg            # Icon — transparent background
│   ├── ghost-icon-rounded.svg    # Icon — dark rounded square bg
│   ├── ghost-icon-circle.svg     # Icon — dark circle bg
│   ├── ghost-favicon.svg         # Simplified icon for 16px
│   ├── ghost-tray-icon.svg       # System tray (white, simplified)
│   ├── ghost-mono-white.svg      # Monochrome white
│   ├── ghost-mono-dark.svg       # Monochrome dark
│   ├── ghost-wordmark-dark.svg   # Logo + "Ghost" text (dark bg)
│   ├── ghost-wordmark-light.svg  # Logo + "Ghost" text (light bg)
│   └── ghost-og-card.svg         # Social media card (1200×630)
├── png/                          # Generated PNGs (all sizes)
├── icons/                        # Additional icon exports
├── social/                       # Social media assets
│   ├── github-avatar.png         # 500×500 GitHub avatar
│   ├── og-card.png               # 1200×630 Open Graph
│   └── og-icon.png               # 630×630 fallback
├── scripts/
│   └── generate-icons.sh         # Asset generation script
└── BRAND_GUIDELINES.md           # This file
```
