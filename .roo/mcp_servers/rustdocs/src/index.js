#!/usr/bin/env node

/**
 * MCP Server for extracting meaningful information from Rust documentation HTML
 * Compatible with Roo Code and other MCP clients
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
  ErrorCode,
  McpError,
} from '@modelcontextprotocol/sdk/types.js';
import { JSDOM } from 'jsdom';
import fetch from 'node-fetch';

class RustDocsParser {
  constructor(htmlString) {
    const dom = new JSDOM(htmlString);
    this.doc = dom.window.document;
  }

  extractAll() {
    return {
      metadata: this.extractMetadata(),
      struct: this.extractStructInfo(),
      methods: this.extractMethods(),
      traits: this.extractTraits(),
      examples: this.extractExamples(),
      fields: this.extractFields(),
      source: this.extractSourceInfo(),
      navigation: this.extractNavigation()
    };
  }

  extractMetadata() {
    const title = this.doc.querySelector('title')?.textContent?.trim();
    const crateInfo = this.doc.querySelector('.crate')?.textContent?.trim();
    const version = this.doc.querySelector('.version')?.textContent?.trim();
    const breadcrumb = this.doc.querySelector('.location')?.textContent?.trim();
    
    const titleMatch = title?.match(/(.+?)\s+in\s+(.+?)\s+-\s+Rust/);
    const itemName = titleMatch?.[1];
    const cratePath = titleMatch?.[2];

    return {
      title,
      itemName,
      cratePath,
      crateInfo,
      version,
      breadcrumb
    };
  }

  extractStructInfo() {
    const structName = this.doc.querySelector('h1.fqn span.struct, h1 .struct')?.textContent?.trim();
    const structDeclaration = this.doc.querySelector('pre.rust.struct, .item-decl pre')?.textContent?.trim();
    const description = this.extractDescription();
    const stability = this.doc.querySelector('.stability')?.textContent?.trim();

    return {
      name: structName,
      declaration: structDeclaration,
      description,
      stability,
      isStruct: !!structName
    };
  }

  extractDescription() {
    const descriptionDiv = this.doc.querySelector('.docblock');
    if (!descriptionDiv) return null;

    const paragraphs = Array.from(descriptionDiv.querySelectorAll('p'))
      .map(p => p.textContent.trim())
      .filter(text => text.length > 0);

    return {
      raw: descriptionDiv.textContent?.trim(),
      paragraphs,
      html: descriptionDiv.innerHTML
    };
  }

  extractMethods() {
    const methods = [];

    // 1) Legacy method blocks (older docs.rs layouts / impl item blocks)
    const methodSections = Array.from(
      this.doc.querySelectorAll('#methods .method, #implementations .method, .impl-items .method')
    );
    methodSections.forEach(methodEl => {
      const signatureEl = methodEl.querySelector('.method-signature, code, pre.rust, pre, .rust');
      const signature = signatureEl?.textContent?.trim() || null;
      const name =
        methodEl.querySelector('h4 code, h3 code, .method-header code')?.textContent?.trim() ||
        (methodEl.getAttribute('id') || '').replace(/^method\./, '') ||
        null;
      const description = methodEl.querySelector('.docblock')?.textContent?.trim() || null;
      const isPublic = !methodEl.classList.contains('hidden');

      if (signature || name) {
        methods.push({
          name,
          signature,
          description,
          isPublic
        });
      }
    });

    // 2) Anchor-based method entries: find anchors by scanning the sidebar for method links first.
    // This is more reliable on docs.rs where navigation lists "method.*" anchors.
    const sidebarMethodLinks = Array.from(
      this.doc.querySelectorAll('.sidebar a[href^="#method."], a[href^="#method."]')
    ).map(a => a.getAttribute('href')?.replace(/^#/, '')).filter(Boolean);

    // Deduplicate
    const anchorIds = Array.from(new Set(sidebarMethodLinks));

    // Also include any in-document anchors not present in sidebar
    const inlineAnchors = Array.from(this.doc.querySelectorAll('[id^="method."]')).map(a => a.getAttribute('id'));
    inlineAnchors.forEach(id => { if (id && !anchorIds.includes(id)) anchorIds.push(id); });

    anchorIds.forEach(id => {
      if (!id) return;
      const name = id.replace(/^method\./, '');

      // Skip duplicates by name
      if (methods.some(m => m.name === name)) return;

      let signature = null;
      let description = null;
      let isPublic = true;

      // Locate the anchor element by id
      const anchor = this.doc.getElementById(id);
      let startEl = anchor || this.doc.querySelector(`[id="${id}"]`);

      // If not found, try to find an element with href to that anchor
      if (!startEl) {
        const link = this.doc.querySelector(`a[href="#${id}"]`);
        startEl = link ? link.parentElement || link : null;
      }

      if (startEl) {
        // Look for signature in the next few siblings or inside the same parent
        let el = startEl.nextElementSibling;
        let attempts = 0;
        while (el && attempts < 12) {
          // direct pre/code tag case
          const tag = el.tagName ? el.tagName.toLowerCase() : '';
          if ((tag === 'pre' || tag === 'code') && el.textContent && el.textContent.trim().length > 0) {
            signature = el.textContent.trim();
            break;
          }

          // nested pre/code inside the element
          const pre = (el.querySelector && el.querySelector('pre.rust, pre, code, .rust')) || null;
          if (pre && pre.textContent && pre.textContent.trim().length > 0) {
            signature = pre.textContent.trim();
            break;
          }

          // sometimes the signature is inline in the element text
          if (el.textContent && /(?:fn|pub|->|impl|struct|enum)\b/.test(el.textContent)) {
            signature = el.textContent.trim();
            break;
          }

          el = el.nextElementSibling;
          attempts++;
        }

        // If still no signature, attempt to find a nearby <pre class="rust"> anywhere close
        if (!signature) {
          const nearbyPre = startEl.closest('section, div, main')?.querySelector('pre.rust, pre, code, .rust');
          if (nearbyPre && nearbyPre.textContent && nearbyPre.textContent.trim().length > 0) {
            signature = nearbyPre.textContent.trim();
          }
        }

        // Description: look for a following .docblock or the parent .docblock
        const docblock =
          startEl.parentElement?.querySelector?.('.docblock') ||
          startEl.querySelector?.('.docblock') ||
          this.doc.querySelector(`#${id} ~ .docblock`) ||
          startEl.nextElementSibling?.querySelector?.('.docblock');
        if (docblock) description = docblock.textContent?.trim();
      }

      methods.push({
        name: name || null,
        signature,
        description,
        isPublic
      });
    });

    // 3) Final fallback: if no methods found, return the names from navigation "Methods" section
    if (methods.length === 0) {
      const navMethodItems = Array.from(this.doc.querySelectorAll('.sidebar a[href^="#method."], .sidebar a[href*="method."]'))
        .map(a => ({
          name: (a.getAttribute('href') || '').replace(/^#method\./, '')
        }))
        .filter(i => i.name);
      navMethodItems.forEach(item => {
        if (!methods.some(m => m.name === item.name)) {
          methods.push({ name: item.name, signature: null, description: null, isPublic: true });
        }
      });
    }

    return methods;
  }

  extractTraits() {
    const traits = [];
    const traitSections = this.doc.querySelectorAll('#trait-implementations .impl, #synthetic-implementations .impl');

    traitSections.forEach(traitEl => {
      const traitName = traitEl.querySelector('.impl-header, .code-header')?.textContent?.trim();
      const methods = Array.from(traitEl.querySelectorAll('.method'))
        .map(method => ({
          name: method.querySelector('code')?.textContent?.trim(),
          signature: method.querySelector('.method-signature')?.textContent?.trim()
        }));

      if (traitName) {
        traits.push({
          name: traitName,
          methods
        });
      }
    });

    return traits;
  }

  extractExamples() {
    const examples = [];
    const codeBlocks = this.doc.querySelectorAll('.docblock pre code, .example-wrap pre code');

    codeBlocks.forEach((codeEl, index) => {
      const code = codeEl.textContent?.trim();
      const language = codeEl.className?.match(/language-(\w+)/)?.[1] || 'rust';
      const parentSection = codeEl.closest('.docblock')?.previousElementSibling?.textContent?.trim();

      if (code && code.length > 10) {
        examples.push({
          index,
          code,
          language,
          context: parentSection
        });
      }
    });

    return examples;
  }

  extractFields() {
    const fields = [];
    const fieldElements = this.doc.querySelectorAll('#fields .field, .variants .variant, #structfields .structfield');

    fieldElements.forEach(fieldEl => {
      const name = fieldEl.querySelector('code, .structfield-name')?.textContent?.trim();
      const type = fieldEl.querySelector('.type')?.textContent?.trim();
      const description = fieldEl.querySelector('.docblock')?.textContent?.trim();
      const isPublic = !fieldEl.classList.contains('hidden');

      if (name) {
        fields.push({
          name,
          type,
          description,
          isPublic
        });
      }
    });

    return fields;
  }

  extractSourceInfo() {
    const sourceLink = this.doc.querySelector('.source, [title="goto source code"]')?.getAttribute('href');
    const sourceText = this.doc.querySelector('.source, [title="goto source code"]')?.textContent?.trim();

    return {
      link: sourceLink ? (sourceLink.startsWith('http') ? sourceLink : `https://docs.rs${sourceLink}`) : null,
      text: sourceText
    };
  }

  extractNavigation() {
    const sidebar = Array.from(this.doc.querySelectorAll('.sidebar a'))
      .map(link => ({
        text: link.textContent?.trim(),
        href: link.getAttribute('href'),
        isActive: link.classList.contains('current')
      }))
      .filter(item => item.text);
 
    const breadcrumbs = Array.from(this.doc.querySelectorAll('.location a'))
      .map(link => ({
        text: link.textContent?.trim(),
        href: link.getAttribute('href')
      }));
 
    return {
      sidebar,
      breadcrumbs
    };
  }
 
  /**
   * extractCrateContents
   *
   * Returns a normalized list of top-level navigation items (modules, structs,
   * enums, traits, functions) present in the crate documentation sidebar.
   *
   * The method attempts to make absolute URLs for sidebar links so callers can
   * fetch the corresponding documentation pages directly.
   */
  extractCrateContents() {
    const sidebar = Array.from(this.doc.querySelectorAll('.sidebar a'))
      .map(link => ({
        text: link.textContent?.trim(),
        href: link.getAttribute('href'),
        isActive: link.classList.contains('current')
      }))
      .filter(item => item.text);
 
    // Normalize hrefs to absolute docs.rs URLs when possible.
    const contents = sidebar.map(item => {
      let href = item.href || null;
      if (href && !href.startsWith('http')) {
        // docs.rs links are often relative; make them absolute for consumer convenience
        href = href.startsWith('/') ? `https://docs.rs${href}` : `https://docs.rs/${href.replace(/^\.\//, '')}`;
      }
      return {
        text: item.text,
        href,
        isActive: item.isActive
      };
    });
 
    return { contents };
  }

  getSummary() {
    const metadata = this.extractMetadata();
    const struct = this.extractStructInfo();
    const methods = this.extractMethods();
    const traits = this.extractTraits();

    return {
      name: metadata.itemName || struct.name,
      type: struct.isStruct ? 'struct' : 'item',
      description: struct.description?.paragraphs?.[0] || 'No description available',
      methodCount: methods.length,
      traitCount: traits.length,
      crate: metadata.cratePath,
      version: metadata.version
    };
  }
}

class RustDocsMcpServer {
  constructor() {
    this.server = new Server(
      {
        name: 'rust-docs-parser',
        version: '1.0.0',
      },
      {
        capabilities: {
          tools: {},
        },
      }
    );

    this.setupToolHandlers();
    this.setupErrorHandling();
  }

  setupToolHandlers() {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => ({
      tools: [
        {
          name: 'parse_rust_docs',
          description: 'Extract meaningful information from Rust documentation HTML pages (docs.rs)',
          inputSchema: {
            type: 'object',
            properties: {
              url: {
                type: 'string',
                description: 'URL of the Rust documentation page to parse',
              },
              html: {
                type: 'string',
                description: 'HTML content to parse directly (alternative to URL)',
              },
              format: {
                type: 'string',
                enum: ['full', 'summary'],
                default: 'full',
                description: 'Output format: "full" for all information or "summary" for key details only',
              },
            },
            oneOf: [
              { required: ['url'] },
              { required: ['html'] }
            ],
          },
        },
        {
          name: 'get_rust_docs_summary',
          description: 'Get a concise summary of Rust documentation',
          inputSchema: {
            type: 'object',
            properties: {
              url: {
                type: 'string',
                description: 'URL of the Rust documentation page',
              },
              html: {
                type: 'string',
                description: 'HTML content to parse directly',
              },
            },
            oneOf: [
              { required: ['url'] },
              { required: ['html'] }
            ],
          },
        },
        {
          name: 'extract_rust_methods',
          description: 'Extract only method information from Rust documentation',
          inputSchema: {
            type: 'object',
            properties: {
              url: {
                type: 'string',
                description: 'URL of the Rust documentation page',
              },
              html: {
                type: 'string',
                description: 'HTML content to parse directly',
              },
            },
            oneOf: [
              { required: ['url'] },
              { required: ['html'] }
            ],
          },
        },
        {
          name: 'list_crate_contents',
          description: 'List top-level modules/items in a crate (navigation/sidebar) and return absolute links where possible',
          inputSchema: {
            type: 'object',
            properties: {
              url: {
                type: 'string',
                description: 'URL of the crate root page or any docs.rs page',
              },
              html: {
                type: 'string',
                description: 'HTML content to parse directly',
              },
            },
            oneOf: [
              { required: ['url'] },
              { required: ['html'] }
            ],
          },
        },
      ],
    }));

    this.server.setRequestHandler(CallToolRequestSchema, async (request) => {
      try {
        const { name, arguments: args } = request.params;

        let htmlContent;
        if (args.url) {
          const response = await fetch(args.url);
          if (!response.ok) {
            throw new Error(`Failed to fetch URL: ${response.status} ${response.statusText}`);
          }
          htmlContent = await response.text();
        } else if (args.html) {
          htmlContent = args.html;
        } else {
          throw new Error('Either url or html parameter is required');
        }

        const parser = new RustDocsParser(htmlContent);

        switch (name) {
          case 'parse_rust_docs':
            const result = args.format === 'summary' ? parser.getSummary() : parser.extractAll();
            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify(result, null, 2),
                },
              ],
            };

          case 'get_rust_docs_summary':
            const summary = parser.getSummary();
            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify(summary, null, 2),
                },
              ],
            };

          case 'extract_rust_methods':
            const methods = parser.extractMethods();
            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify(methods, null, 2),
                },
              ],
            };
 
          case 'list_crate_contents':
            const crateContents = parser.extractCrateContents();
            return {
              content: [
                {
                  type: 'text',
                  text: JSON.stringify(crateContents, null, 2),
                },
              ],
            };

          default:
            throw new McpError(
              ErrorCode.MethodNotFound,
              `Unknown tool: ${name}`
            );
        }
      } catch (error) {
        throw new McpError(
          ErrorCode.InternalError,
          `Tool execution failed: ${error.message}`
        );
      }
    });
  }

  setupErrorHandling() {
    this.server.onerror = (error) => {
      console.error('[MCP Error]', error);
    };

    process.on('SIGINT', async () => {
      await this.server.close();
      process.exit(0);
    });
  }

  async run() {
    const transport = new StdioServerTransport();
    await this.server.connect(transport);
    console.error('Rust Documentation MCP Server running on stdio');
  }
}

// Start the server
const server = new RustDocsMcpServer();
server.run().catch(console.error);