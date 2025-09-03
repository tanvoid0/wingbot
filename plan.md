# Wingbot Migration Plan: Flutter → Tauri + Svelte

## Overview
Migrate from Flutter client to a modern Tauri + Svelte desktop application while keeping the existing Python server and Flutter client in their respective folders.

## Project Structure
```
wingbot/
├── wingbot_client/          # Existing Flutter client (kept for reference)
├── wingbot_server/          # Existing Python server (kept)
├── wingbot/         # NEW: Tauri + Svelte desktop app
│   ├── src-tauri/          # Rust backend
│   ├── src/                # Svelte frontend
│   ├── package.json
│   └── tailwind.config.js
└── plan.md                  # This file
```

## Migration Steps

### Phase 1: Setup Tauri + Svelte Project
1. ✅ Create plan.md
2. ✅ Initialize SvelteKit project with modern tooling
   - ✅ SvelteKit (latest) - Full-stack framework with routing and API endpoints
   - ✅ TailwindCSS - Utility-first CSS framework for rapid UI development
   - ✅ ESLint + Prettier - Code quality and formatting tools
   - ✅ Vitest - Unit and component testing framework
   - ✅ Playwright - End-to-end testing framework
   - ✅ SQLite + better-sqlite3 - Local database for desktop app
   - ✅ Storybook - Component development and testing environment
3. ✅ Configure latest versions:
   - ✅ Tauri (latest) - Added and configured
   - ✅ Svelte (latest) - Included with SvelteKit
4. ✅ Set up project structure with modular components

### Phase 2: UI Component Migration
1. Create reusable UI kit using shadcn/ui
2. Migrate Flutter widgets to Svelte components:
   - Dashboard screen
   - Champion select screen
   - Game info cards
   - Abilities cards
   - Player cards
   - Settings cards
3. Implement navigation rail (as per user preference)
4. Ensure modular and reusable component architecture

### Phase 3: API Integration
1. Integrate with existing Python server
2. Maintain API endpoint structure (/api/v1)
3. Implement proper error handling
4. Add loading states and user feedback

### Phase 4: Features & Polish
1. Implement all existing functionality
2. Add modern desktop-specific features
3. Ensure responsive design
4. Add proper TypeScript types
5. Implement proper state management

## Current Progress - SvelteKit Template Setup

### What We're Getting from the Template:
- **SvelteKit** - Modern full-stack framework perfect for Tauri integration
- **TailwindCSS** - Rapid UI development with utility classes
- **ESLint + Prettier** - Code quality and consistent formatting
- **Vitest** - Testing framework for both unit and component tests
- **Playwright** - E2E testing for comprehensive app testing
- **SQLite + better-sqlite3** - Local database perfect for desktop apps
- **Storybook** - Component development environment for UI consistency

### Next Steps After Template Setup:
1. ✅ Add Tauri to the SvelteKit project
2. ✅ Configure Tauri for desktop app packaging
3. ✅ Set up the project structure for WingBot components
4. 🔄 Begin migrating Flutter widgets to Svelte

## Technical Requirements
- Use CLI commands for project initialization
- Use shadcn/ui for component library
- Avoid custom CSS, use Tailwind configuration
- Keep components modular and reusable
- Use latest available libraries
- Maintain clean, non-duplicated code

## User Preferences to Follow
- Navigation rail instead of tabs for settings
- Avoid custom CSS, prefer Tailwind
- Modular, reusable components
- Clean, non-duplicated code
- Use CLI commands for setup
- Prefer latest library versions

## Success Criteria
- [x] SvelteKit project runs successfully with all tooling
- [x] Tauri integration works properly
- [ ] All existing Flutter functionality is replicated
- [ ] UI is modern and responsive
- [ ] Components are modular and reusable
- [ ] Integration with existing server works
- [ ] Code follows user preferences

## Template Benefits for WingBot
- **SvelteKit routing** - Perfect for multi-screen desktop app navigation
- **API routes** - Can build backend endpoints right in the app
- **Component testing** - Vitest + Storybook for reliable UI development
- **Local database** - SQLite for storing game data and user preferences
- **Modern tooling** - Latest versions of all dependencies
- **Desktop-ready** - Optimized for Tauri desktop app performance

## Backend Development Progress

### ✅ Tauri Backend Setup Complete
- **Rust Backend Structure**: Created modular backend with proper separation of concerns
- **Game Management**: Implemented game state management, champion suggestions, and ability recommendations
- **System Integration**: Added League client detection and system information gathering
- **AI Service Integration**: Created service layer for communicating with existing Python AI backend
- **Configuration Management**: Built comprehensive configuration system with file persistence
- **API Commands**: Implemented 15+ Tauri commands for frontend-backend communication

### 🔧 Backend Architecture
```
src-tauri/
├── src/
│   ├── lib.rs          # Main application and command handlers
│   ├── game.rs         # Game state and champion management
│   ├── system.rs       # System info and League client detection
│   ├── ai_service.rs   # AI backend integration
│   └── config.rs       # Configuration management
├── Cargo.toml          # Dependencies and build configuration
└── tauri.conf.json     # Tauri app configuration
```

### 🚀 Ready for Frontend Integration
The backend now provides:
- Game state management and updates
- Champion selection recommendations
- Ability usage suggestions
- System information and League client status
- AI service communication
- Configuration persistence and management
- Proper error handling and async operations
