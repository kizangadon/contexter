import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { resolve } from 'path';
import { defineConfig } from 'vite';

/**
 * Vite plugin to inject Content-Security-Policy into the HTML
 * during dev (the static index.html <meta> covers production builds).
 */
function cspPlugin(): import('vite').Plugin {
  const CSP =
    "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; frame-ancestors 'none'; base-uri 'self'";
  return {
    name: 'csp',
    transformIndexHtml(html) {
      return html.replace(
        '</head>',
        `  <meta http-equiv="Content-Security-Policy" content="${CSP}" />\n</head>`,
      );
    },
  };
}

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss(), cspPlugin()],
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
    },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://localhost:8051',
        changeOrigin: true,
      },
    },
  },
  build: {
    chunkSizeWarningLimit: 300,
    modulePreload: {
      resolveDependencies: (_url, deps, _context) => {
        // Don't preload vendor-charts on every page (386KB recharts)
        return deps.filter(dep => !dep.includes('vendor-charts'));
      },
    },
    rolldownOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes('node_modules/react/') || id.includes('node_modules/react-dom/') || id.includes('node_modules/react-router/')) return 'vendor-react';
          if (id.includes('node_modules/@tanstack/react-query/')) return 'vendor-query';
          if (id.includes('node_modules/framer-motion/')) return 'vendor-fm';
          if (id.includes('node_modules/recharts/')) return 'vendor-charts';
          if (id.includes('node_modules/lucide-react/')) return 'vendor-icons';
        },
      },
    },
  },
});
