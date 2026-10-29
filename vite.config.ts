import { unplugin as stylexPlugin } from "@stylexjs/unplugin";
import { fileURLToPath } from "node:url";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
// @ts-expect-error
import process from "node:process";
const host = process.env.TAURI_DEV_HOST;

const stylexDevUrlsOnWindows = (): Plugin => ({
  name: "stylex-dev-urls-on-windows",
  apply: "serve",
  enforce: "post",
  transformIndexHtml: {
    order: "post",
    handler: (html) =>
      html.replace(/(src|href)="\\+(@id\\+)?virtual:stylex([^"]*)"/g, (_m, attr, id, rest) =>
        `${attr}="/${id ? "@id/" : ""}virtual:stylex${rest}"`),
  },
});

const withoutStylexImportWalk = (p: Plugin | Plugin[]): Plugin[] =>
  (Array.isArray(p) ? p : [p]).map((plugin) => {
    const hook = plugin.transform;
    if (typeof hook !== "function") return plugin;
    return {
      ...plugin,
      transform(this: any, ...args: any[]) {
        const ctx = new Proxy(this, {
          get: (t, k) => (k === "meta" ? { ...t.meta, watchMode: false } : Reflect.get(t, k)),
        });
        return (hook as any).apply(ctx, args);
      },
    };
  });

export default defineConfig(() => ({
  plugins: [
    ...(process.env.VITEST
      ? []
      : withoutStylexImportWalk(
          stylexPlugin.vite({
            aliases: {
              "@/*": [fileURLToPath(new URL("src/*", import.meta.url))],
            },
            unstable_moduleResolution: { type: "commonJS" },
          }) as Plugin,
        )),
    stylexDevUrlsOnWindows(),
    react(),
  ],
  resolve: {
    alias: { "@": fileURLToPath(new URL("src", import.meta.url)) },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    target: "chrome123",
  },
}));
