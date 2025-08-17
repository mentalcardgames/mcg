#!/usr/bin/env node
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import axios from "axios";
import { load } from "cheerio";
import { z } from "zod";

const server = new McpServer({ name: "rustdocs", version: "0.1.0" });

const cratesIo = axios.create({ baseURL: "https://crates.io/api/v1" });
const docsRsBase = "https://docs.rs";

async function fetchCrateInfo(crateName) {
  try {
    const res = await cratesIo.get(`/crates/${encodeURIComponent(crateName)}`);
    return res.data.crate;
  } catch (err) {
    if (err.response && err.response.status === 404) return null;
    throw err;
  }
}

function buildDocsRsBaseUrl(crateName, version) {
  if (version) {
    return `${docsRsBase}/crate/${encodeURIComponent(crateName)}/${encodeURIComponent(version)}`;
  }
  // docs.rs redirects to latest if no version when using /crate/...
  return `${docsRsBase}/crate/${encodeURIComponent(crateName)}`;
}

async function fetchDocsRsPage(crateName, version, path = "") {
  const base = buildDocsRsBaseUrl(crateName, version);
  const url = path ? `${base}/${path}` : `${base}/`;
  const res = await axios.get(url, { validateStatus: null });
  if (res.status >= 400) {
    const err = new Error(`Failed to fetch ${url}: status ${res.status}`);
    err.status = res.status;
    throw err;
  }
  return { url, html: res.data };
}

server.tool(
  "resolve-crate-id",
  { crateName: z.string().describe("Crate name, e.g. serde") },
  async ({ crateName }) => {
    const crate = await fetchCrateInfo(crateName);
    if (!crate) {
      return {
        content: [{ type: "text", text: `Crate not found: ${crateName}` }],
        isError: true,
      };
    }

    const latest = crate.max_version;
    return {
      content: [
        {
          type: "text",
          text: JSON.stringify(
            {
              crate: crate.name,
              description: crate.description,
              latest_version: latest,
              repository: crate.repository,
            },
            null,
            2
          ),
        },
      ],
    };
  }
);

server.tool(
  "get-crate-docs",
  {
    crate: z.string().describe("Crate name, e.g. serde"),
    version: z.string().optional().describe("Optional version, default to latest"),
  },
  async ({ crate, version }) => {
    const info = await fetchCrateInfo(crate);
    if (!info) {
      return {
        content: [{ type: "text", text: `Crate not found: ${crate}` }],
        isError: true,
      };
    }

    const ver = version || info.max_version;
    try {
      const { url, html } = await fetchDocsRsPage(crate, ver, "");
      const $ = load(html);
      // Try to extract the crate documentation summary from the main content
      const title = $("h1").first().text().trim() || `${crate} ${ver}`;
      const firstPara = $("#main p").first().text().trim() || $(".docblock p").first().text().trim();
      const summary = firstPara || "No summary available in docs.rs";

      return {
        content: [
          { type: "text", text: `crate: ${crate}` },
          { type: "text", text: `version: ${ver}` },
          { type: "text", text: `url: ${url}` },
          { type: "text", text: `title: ${title}` },
          { type: "text", text: `summary: ${summary}` },
        ],
      };
    } catch (err) {
      return {
        content: [{ type: "text", text: `Error fetching docs: ${err.message}` }],
        isError: true,
      };
    }
  }
);

server.tool(
  "get-item-docs",
  {
    crate: z.string().describe("Crate name, e.g. serde"),
    version: z.string().optional().describe("Optional version, default to latest"),
    item: z.string().describe("Item name or path, e.g. serde::ser::Serializer or MyStruct"),
  },
  async ({ crate, version, item }) => {
    const info = await fetchCrateInfo(crate);
    if (!info) {
      return {
        content: [{ type: "text", text: `Crate not found: ${crate}` }],
        isError: true,
      };
    }

    const ver = version || info.max_version;
    try {
      // First try direct path resolution: docs.rs uses paths like module/index.html or type.Struct.html
      // Use the docs.rs search page to locate the best match
      const searchUrl = `${buildDocsRsBaseUrl(crate, ver)}/?search=${encodeURIComponent(item)}`;
      const searchRes = await axios.get(searchUrl, { validateStatus: null });
      if (searchRes.status >= 400) {
        throw new Error(`Search failed: status ${searchRes.status}`);
      }
      const $search = load(searchRes.data);

      // Attempt to find the first search result link
      let link = null;
      // docs.rs often renders search results inside elements with class 'search' or 'matches'
      const candidateAnchors = $search("a").toArray();
      for (const a of candidateAnchors) {
        const $a = $search(a);
        const href = $a.attr("href");
        const text = $a.text();
        if (!href) continue;
        // Prefer anchors that contain the item string or end with .html
        if (text.includes(item) || href.endsWith(".html")) {
          link = href;
          break;
        }
      }

      if (!link) {
        // Fallback: try to construct a type page for struct/enums: type.{Item}.html
        const typePage = `/${encodeURIComponent(crate)}/${encodeURIComponent(ver)}/type.${encodeURIComponent(item)}.html`;
        const fallbackUrl = `${docsRsBase}${typePage}`;
        const fallbackRes = await axios.get(fallbackUrl, { validateStatus: null });
        if (fallbackRes.status < 400) {
          const $f = load(fallbackRes.data);
          const docHtml = $f("#main").html() || $f(".docblock").html() || fallbackRes.data;
          return {
            content: [{ type: "text", text: `url: ${fallbackUrl}` }, { type: "text", text: docHtml }],
          };
        }

        return {
          content: [{ type: "text", text: `Item not found in docs.rs search results for "${item}"` }],
          isError: true,
        };
      }

      // Normalize relative links
      if (link.startsWith("/")) {
        link = `${docsRsBase}${link}`;
      } else if (!link.startsWith("http")) {
        const base = buildDocsRsBaseUrl(crate, ver);
        link = `${base}/${link}`;
      }

      const itemRes = await axios.get(link, { validateStatus: null });
      if (itemRes.status >= 400) {
        throw new Error(`Failed to fetch item page: ${itemRes.status}`);
      }

      const $item = load(itemRes.data);
      // Extract the main documentation block
      const docHtml = $item("#main").html() || $item(".docblock").html() || itemRes.data;
      // Optionally extract signatures or code examples
      const signatures = $item(".rustdoc-signature, .in-band, code").first().text().trim();

      const texts = [
        { type: "text", text: `url: ${link}` },
        { type: "text", text: `signature: ${signatures || "n/a"}` },
        { type: "text", text: docHtml },
      ];

      return { content: texts };
    } catch (err) {
      return {
        content: [{ type: "text", text: `Error fetching item docs: ${err.message}` }],
        isError: true,
      };
    }
  }
);

// Start the server
(async () => {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("rustdocs MCP server running on stdio");
})().catch((err) => {
  console.error("rustdocs server failed:", err);
  process.exit(1);
});