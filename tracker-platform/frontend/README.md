# Tracker Platform Frontend

Modern, feature-rich SvelteKit 2.0 frontend for the Tracker Platform.

## Features

- **SvelteKit 2.0** - Modern web framework
- **TypeScript** - Type-safe development
- **TailwindCSS 4** - Utility-first styling
- **GraphQL (urql)** - Efficient data fetching
- **WebSocket Support** - Real-time features
- **5 Beautiful Themes**:
  - Dark - Modern dark with blue/purple accents
  - Grey - Professional neutral grey tones
  - Light - Clean white with subtle colors
  - Frutiger Aero - Glossy Windows Vista/7 aesthetic
  - Global Coffeehouse - Warm, cozy coffee shop vibes
- **PWA Support** - Installable web app
- **Responsive Design** - Mobile-first approach
- **Real-time Chat** - WebSocket-powered messaging

## Getting Started

### Prerequisites

- Node.js 18+ and npm/pnpm/yarn

### Installation

```bash
# Install dependencies
npm install

# Copy environment variables
cp .env.example .env

# Update .env with your API endpoints
```

### Development

```bash
# Start development server
npm run dev

# Open http://localhost:3000
```

### Building

```bash
# Build for production
npm run build

# Preview production build
npm run preview
```

## Project Structure

```
src/
├── lib/
│   ├── components/
│   │   ├── common/      # Reusable components
│   │   ├── layout/      # Layout components
│   │   ├── torrent/     # Torrent components
│   │   └── user/        # User components
│   ├── graphql/
│   │   ├── client.ts    # GraphQL client setup
│   │   ├── queries.ts   # GraphQL queries
│   │   ├── mutations.ts # GraphQL mutations
│   │   └── subscriptions.ts
│   ├── stores/
│   │   ├── auth.ts      # Authentication store
│   │   ├── theme.ts     # Theme management
│   │   ├── notifications.ts
│   │   └── websocket.ts # WebSocket store
│   └── utils/           # Utility functions
├── routes/
│   ├── +layout.svelte   # Root layout
│   ├── +page.svelte     # Homepage
│   ├── login/
│   ├── register/
│   ├── torrents/
│   ├── torrent/[id]/
│   ├── forums/
│   ├── chat/
│   ├── search/
│   ├── stats/
│   └── user/
└── app.css              # Global styles

```

## Theme System

The application includes 5 switchable themes:

1. **Dark Theme** - Default modern dark theme
2. **Grey Theme** - Professional neutral theme
3. **Light Theme** - Clean light theme
4. **Frutiger Aero** - Vista-era glossy theme with glass effects
5. **Global Coffeehouse** - Warm, cozy theme with coffee tones

Switch themes using the theme selector in the header.

## GraphQL Integration

The app uses urql for GraphQL with support for:

- Queries
- Mutations
- Subscriptions (real-time updates)
- WebSocket connections

## Real-time Features

- Live chat with typing indicators
- Real-time torrent stats updates
- Live notifications
- WebSocket fallback support

## Environment Variables

```env
PUBLIC_API_URL=http://localhost:4000
PUBLIC_GRAPHQL_URL=http://localhost:4000/graphql
PUBLIC_GRAPHQL_WS_URL=ws://localhost:4000/graphql
PUBLIC_WS_URL=ws://localhost:4000
```

## Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run check` - Run type checking
- `npm run lint` - Run linter
- `npm run format` - Format code

## License

MIT
