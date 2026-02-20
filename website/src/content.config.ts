import { docsLoader } from "@astrojs/starlight/loaders";
import { docsSchema } from "@astrojs/starlight/schema";
import { defineCollection } from "astro:content";

export const collections = {
  docs: defineCollection({
    loader: docsLoader({
      // Preserve original file paths as slugs (don't normalize hyphens/case)
      generateId({ entry }) {
        return entry.replace(/\.(md|mdx)$/, "");
      },
    }),
    schema: docsSchema(),
  }),
};
