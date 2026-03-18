# ShowMe

Open-source, macOS-native AI-powered screen guidance tool.

Press a hotkey, ask a question, and ShowMe captures your screen, sends it to an AI vision model, and renders visual overlays (highlights, arrows, step callouts) directly on your screen — showing you exactly where to click and what to do.

## Overview

- **Global hotkey** (Cmd+Shift+H) activates a minimal floating input bar
- **Screen capture** grabs what's on your display
- **Privacy-first consent** — 5-second preview before anything leaves your device
- **AI vision analysis** via OpenRouter (Claude, GPT-4o, and more)
- **Visual overlays** drawn on a transparent click-through window

## Tech Stack

- [Tauri v2](https://tauri.app/) — lightweight native shell (Rust backend + WebView frontend)
- [React 19](https://react.dev/) + [TypeScript](https://www.typescriptlang.org/) — frontend UI
- [Vite](https://vitejs.dev/) — fast build tooling
- [OpenRouter](https://openrouter.ai/) — AI model routing

## Project Structure

```
show-me/
├── src/               # React frontend
│   ├── App.tsx
│   └── main.tsx
├── src-tauri/         # Rust backend
│   ├── src/
│   │   └── lib.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── public/            # Static assets
├── index.html
├── vite.config.ts
└── tsconfig.json
```

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 1.80+)
- [Node.js](https://nodejs.org/) (v22+)
- [pnpm](https://pnpm.io/)
- macOS 11+

### Setup

```bash
pnpm install
```

### Run dev server

```bash
pnpm tauri dev
```

### Build

```bash
pnpm tauri build
```

### Lint

```bash
pnpm lint
```

## License

MIT
