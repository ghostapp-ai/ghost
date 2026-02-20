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
        "The Private Agent OS for Desktop & Mobile — local-first AI that never sends your data to the cloud.",
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
      ],
      favicon: "/favicon.svg",
      head: [
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
            name: "twitter:card",
            content: "summary_large_image",
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
