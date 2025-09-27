import { defineConfig } from "vitepress";
import { generateSidebar } from "vitepress-sidebar";

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "beaverCDS Docs",
  description: "Next-generation CTF deployment framework",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: "Setup", link: "guides/infra-quickstart" },
      {
        text: "Guides",
        items: [
          { text: "Deployment Quickstart", link: "for-sysadmins/quickstart" },
          { text: "Add new challenge", link: "for-authors/quickstart" },
        ],
      },

      {
        text: "Infrastructure Setup",
        items: [
          { text: "Quickstart", link: "/for-sysadmins/quickstart" },
          { text: "Install", link: "/for-sysadmins/install" },
          { text: "Config Reference", link: "/for-sysadmins/config" },
          { text: "Architecture", link: "/for-sysadmins/architecture" },
        ],
      },
      {
        text: "Challenge Authors",
        items: [
          { text: "Challenge Quickstart", link: "/for-authors/quickstart" },
          {
            text: "Challenge Config Reference",
            link: "/for-authors/challenge-config",
          },
        ],
      },
    ],

    // auto generate sidebar from directory structure, via vitepress-sidebar
    sidebar: generateSidebar({
      documentRootPath: "./",
      // pull title from markdown not filename
      useTitleFromFileHeading: true,
      useTitleFromFrontmatter: true,
      keepMarkdownSyntaxFromTitle: true,
      useFolderTitleFromIndexFile: true,
      // transform name to sentence case
      hyphenToSpace: true,
      underscoreToSpace: true,
      // capitalizeEachWords: true,

      sortFolderTo: "bottom",
      sortMenusByFrontmatterOrder: true,
    }),

    socialLinks: [
      { icon: "github", link: "https://github.com/osusec/beavercds-ng" },
    ],
  },

  // disable interpolation of {{ and }} in markdown
  markdown: {
    config(md) {
      const defaultCodeInline = md.renderer.rules.code_inline!;
      md.renderer.rules.code_inline = (tokens, idx, options, env, self) => {
        tokens[idx].attrSet("v-pre", "");
        return defaultCodeInline(tokens, idx, options, env, self);
      };
    },
  },
});
