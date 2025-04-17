# Create the project directory structure
mkdir -p SmolDesk/{src-tauri,src,docs/{api,user,technical},tests/{unit,integration,e2e}}

# Initialize package.json for the frontend
cd SmolDesk
npm init -y

# Install core frontend dependencies
npm install react react-dom vite @vitejs/plugin-react typescript tailwindcss postcss autoprefixer

# Initialize TypeScript configuration
npx tsc --init

# Initialize Tauri
npm install --save-dev @tauri-apps/api @tauri-apps/cli
npx tauri init

# Configure Git repository
git init
echo "node_modules/" > .gitignore
echo "dist/" >> .gitignore
echo "target/" >> .gitignore
echo ".env" >> .gitignore

# Create initial commit
git add .
git commit -m "Initial project setup for SmolDesk"

# Generate basic Vite + React configuration
cat > vite.config.ts << EOL
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // to make use of \`TAURI_DEBUG\` and other env variables
  // https://tauri.app/v1/api/config/#buildconfig.beforedevcommand
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    // Tauri supports es2021
    target: ['es2021', 'chrome100', 'safari13'],
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
EOL

# Create the base React structure
mkdir -p src/{components,hooks,utils,contexts}

# Create index.html
cat > index.html << EOL
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>SmolDesk - WebRTC Remote Desktop</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
EOL

# Create initial React app
cat > src/main.tsx << EOL
import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
EOL

# Create basic App component
cat > src/App.tsx << EOL
import { useState } from 'react';

function App() {
  const [status, setStatus] = useState('Initializing...');

  return (
    <div className="container mx-auto p-4">
      <h1 className="text-2xl font-bold mb-4">SmolDesk</h1>
      <p className="mb-4">WebRTC-based Remote Desktop for Linux</p>
      <div className="p-4 bg-gray-100 rounded">
        <p>Status: <span className="font-mono">{status}</span></p>
      </div>
    </div>
  );
}

export default App;
EOL

# Create styles file
cat > src/styles.css << EOL
@tailwind base;
@tailwind components;
@tailwind utilities;

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
EOL

# Configure Tailwind
cat > tailwind.config.js << EOL
/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
EOL

cat > postcss.config.js << EOL
module.exports = {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
EOL

# Create initial documentation
cat > docs/README.md << EOL
# SmolDesk Documentation

This directory contains the documentation for SmolDesk, a WebRTC-based remote desktop tool.

## Structure

- [API Documentation](./api/): API references and integration guides
- [User Documentation](./user/): End-user guides and tutorials
- [Technical Documentation](./technical/): Architecture, design decisions, and implementation details

## Development Status

SmolDesk is currently in initial development phase. See the implementation plan for details on the project roadmap.
EOL

# Update package.json with scripts
cat > package.json << EOL
{
  "name": "smoldesk",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0"
  },
  "devDependencies": {
    "@types/react": "^18.2.15",
    "@types/react-dom": "^18.2.7",
    "@vitejs/plugin-react": "^4.0.3",
    "autoprefixer": "^10.4.14",
    "postcss": "^8.4.27",
    "tailwindcss": "^3.3.3",
    "typescript": "^5.0.2",
    "vite": "^4.4.4",
    "@tauri-apps/cli": "^1.4.0"
  }
}
EOL

# Generate basic README for the repository
cat > README.md << EOL
# SmolDesk

SmolDesk is a WebRTC-based remote desktop tool for Linux, supporting both X11 and Wayland display servers.

## Features (Planned)

- Low-latency peer-to-peer remote desktop streaming
- Support for X11 and Wayland
- Hardware acceleration (VAAPI/NVENC)
- Multi-monitor support
- Clipboard synchronization
- File transfer
- Secure connections with OAuth2 and encryption

## Development Status

SmolDesk is currently in early development. See the [documentation](./docs/) for more information on the project roadmap.

## Getting Started

### Prerequisites

- Node.js (v16+)
- Rust (latest stable)
- For Linux development:
  - X11 or Wayland
  - FFmpeg development libraries
  - For hardware acceleration:
    - VAAPI development libraries (Intel)
    - NVENC SDK (NVIDIA)

### Installation

1. Clone the repository
2. Install dependencies:
   ```bash
   npm install
   ```
3. Run the development server:
   ```bash
   npm run tauri dev
   ```

## License

[MIT](LICENSE)
EOL
