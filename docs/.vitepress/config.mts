import { defineConfig } from "vitepress";
import { withMermaid } from 'vitepress-plugin-mermaid';

// https://vitepress.dev/reference/site-config
export default withMermaid(
  defineConfig({
    srcDir: './pages',
    lastUpdated: true,
    markdown: {
      math: true,
      lineNumbers: true,
    },
    title: "Mental Card Games",
    description: "Mental Card Games Documentation",
    themeConfig: {
      // https://vitepress.dev/reference/default-theme-config
      editLink: {
        pattern: 'https://github.com/mentalcardgames/mentalcardgames.github.io/edit/main/pages/:path'
      },
      nav: [
        { text: "Home", link: "/" },
        { text: "Lifecycle", link: "/organisational/lifecycle" },
        { text: "Documentation", link: "/project/system-design" },
      ],

      sidebar: [
        {
          text: "The Student Journey",
          items: [
            { text: "Participation Lifecycle", link: "/organisational/lifecycle" },
            { text: "Developer Setup & Rules", link: "/organisational/contribute" },
            { text: "Literature & References", link: "/organisational/literature" },
            { text: "Active Registry (People)", link: "/organisational/people" },
          ],
        },
        {
          text: "Technical Blueprints",
          items: [
            { text: "Overview & Mindmap", link: "/project/" },
            { text: "Vision & Baselines", link: "/project/vision" },
            { text: "System Design", link: "/project/system-design" },
            { text: "Repository Components & Layout", link: "/project/repository-components-layout" },
            { text: "Student Milestones", link: "/project/milestones" },
          ],
        },
        {
          text: "Component Deep Dives",
          items: [
            { text: "Backend Engine", link: "/component/backend" },
            { text: "Frontend Client", link: "/component/frontend" },
            { text: "Text User Interface (TUI)", link: "/component/tui" },
            { text: "Game Engine (FSM)", link: "/component/engine" },
            { text: "QR-Code Protocol", link: "/component/qr-comm" },
            { text: "CGDL Compiler", link: "/component/cgdl" },
            { text: "Poker Game Implementation", link: "/component/poker" },
          ],
        },
      ],
      search: {
        provider: "local",
      },
      socialLinks: [
        { icon: "github", link: "https://github.com/mentalcardgames" },
      ],
    },
  })
);
