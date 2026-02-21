// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";

// https://astro.build/config
export default defineConfig({
  site: "https://ghostapp-ai.github.io",
  base: "/ghost",
  integrations: [
    starlight({
      title: "Ghost",
      description:
        "Ghost is a private, local-first Agent OS for desktop and mobile. The first app implementing the complete 2026 agent protocol stack — MCP, MCP Apps, AG-UI, A2UI, A2A. Run AI agents, search files semantically, and connect to thousands of tools with on-device inference.",
      logo: {
        src: "./src/assets/ghost-icon.svg",
        replacesTitle: false,
      },
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/ghostapp-ai/ghost",
        },
        {
          icon: "x.com",
          label: "X / Twitter",
          href: "https://x.com/ghostapp_ai",
        },
      ],
      favicon: "/favicon.svg",
      head: [
        // Open Graph
        {
          tag: "meta",
          attrs: {
            property: "og:image",
            content: "https://ghostapp-ai.github.io/ghost/og-card.png",
          },
        },
        {
          tag: "meta",
          attrs: {
            property: "og:image:width",
            content: "1200",
          },
        },
        {
          tag: "meta",
          attrs: {
            property: "og:image:height",
            content: "630",
          },
        },
        {
          tag: "meta",
          attrs: {
            property: "og:type",
            content: "website",
          },
        },
        {
          tag: "meta",
          attrs: {
            property: "og:site_name",
            content: "Ghost — The Private Agent OS",
          },
        },
        {
          tag: "meta",
          attrs: {
            property: "og:locale",
            content: "en_US",
          },
        },
        // Twitter Card
        {
          tag: "meta",
          attrs: {
            name: "twitter:card",
            content: "summary_large_image",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "twitter:site",
            content: "@ghostapp_ai",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "twitter:creator",
            content: "@ghostapp_ai",
          },
        },
        // SEO meta
        {
          tag: "meta",
          attrs: {
            name: "keywords",
            content:
              "ghost,agent os,local ai,private ai,mcp,mcp apps,desktop ai,semantic search,vector search,tauri,rust,on-device ai,on-device inference,sovereign ai,open source,privacy first,local first,agentic,tool calling,agent orchestration,protocol hub,generative ui,a2ui,ag-ui,a2a",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "author",
            content: "ghostapp-ai",
          },
        },
        {
          tag: "meta",
          attrs: {
            name: "robots",
            content: "index, follow",
          },
        },
        // JSON-LD Structured Data — SoftwareApplication
        {
          tag: "script",
          attrs: {
            type: "application/ld+json",
          },
          content: JSON.stringify({
            "@context": "https://schema.org",
            "@type": "SoftwareApplication",
            name: "Ghost",
            applicationCategory: "UtilitiesApplication",
            applicationSubCategory: "AI Agent",
            operatingSystem: "Windows, macOS, Linux, Android",
            description:
              "Ghost is a private, local-first Agent OS for desktop and mobile. The first app with the complete 2026 agent protocol stack (MCP, MCP Apps, AG-UI, A2UI, A2A). On-device inference, zero cloud, zero telemetry.",
            url: "https://ghostapp-ai.github.io/ghost",
            downloadUrl:
              "https://github.com/ghostapp-ai/ghost/releases/latest",
            softwareVersion: "0.11.0",
            license: "https://opensource.org/licenses/MIT",
            featureList:
              "Hybrid search (FTS5 + vector), Native AI inference, ReAct agent engine, MCP protocol hub, MCP Apps renderer, AG-UI streaming, A2UI generative UI, A2A multi-agent, Skills system, Zero telemetry, Cross-platform, On-device inference",
            author: {
              "@type": "Organization",
              name: "ghostapp-ai",
              url: "https://github.com/ghostapp-ai",
            },
            offers: {
              "@type": "Offer",
              price: "0",
              priceCurrency: "USD",
            },
            image:
              "https://ghostapp-ai.github.io/ghost/og-card.png",
          }),
        },
        // JSON-LD — Organization for brand knowledge graph
        {
          tag: "script",
          attrs: {
            type: "application/ld+json",
          },
          content: JSON.stringify({
            "@context": "https://schema.org",
            "@type": "Organization",
            name: "ghostapp-ai",
            url: "https://ghostapp-ai.github.io/ghost",
            logo: "https://ghostapp-ai.github.io/ghost/og-card.png",
            sameAs: [
              "https://github.com/ghostapp-ai",
              "https://x.com/ghostapp_ai",
            ],
          }),
        },
        // JSON-LD — WebSite for sitelinks search box
        {
          tag: "script",
          attrs: {
            type: "application/ld+json",
          },
          content: JSON.stringify({
            "@context": "https://schema.org",
            "@type": "WebSite",
            name: "Ghost — The Private Agent OS",
            url: "https://ghostapp-ai.github.io/ghost",
            description:
              "Documentation and downloads for Ghost, the private local-first Agent OS.",
            publisher: {
              "@type": "Organization",
              name: "ghostapp-ai",
              url: "https://github.com/ghostapp-ai",
            },
          }),
        },
        // Theme color for mobile browsers
        {
          tag: "meta",
          attrs: {
            name: "theme-color",
            content: "#7c3aed",
          },
        },
        // Canonical (helps avoid duplicate content with GitHub Pages)
        {
          tag: "link",
          attrs: {
            rel: "canonical",
            href: "https://ghostapp-ai.github.io/ghost/",
          },
        },
      ],
      customCss: ["./src/styles/custom.css"],
      editLink: {
        baseUrl: "https://github.com/ghostapp-ai/ghost/edit/main/website/",
      },
      sidebar: [
        {
          label: "Getting Started",
          items: [
            { label: "Introduction", slug: "guides/introduction" },
            { label: "Installation", slug: "guides/installation" },
            { label: "Quick Start", slug: "guides/quickstart" },
          ],
        },
        {
          label: "Features",
          items: [
            { label: "Search", slug: "features/search" },
            { label: "Native AI", slug: "features/native-ai" },
            { label: "Chat Engine", slug: "features/chat" },
            { label: "Agent Engine", slug: "features/agent" },
            { label: "Protocol Hub", slug: "features/protocols" },
            { label: "Skills System", slug: "features/skills" },
          ],
        },
        {
          label: "Architecture",
          items: [
            { label: "Overview", slug: "architecture/overview" },
            { label: "Database Schema", slug: "architecture/database" },
            { label: "Embedding Engine", slug: "architecture/embeddings" },
            { label: "Multiplatform", slug: "architecture/multiplatform" },
          ],
        },
        {
          label: "Reference",
          items: [
            { label: "Changelog", slug: "reference/changelog" },
            { label: "Roadmap", slug: "reference/roadmap" },
            { label: "Privacy & Security", slug: "reference/privacy" },
            { label: "Contributing", slug: "reference/contributing" },
          ],
        },
      ],
      lastUpdated: true,
    }),
  ],
});
