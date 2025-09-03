# Wingbot Migration Plan: Flutter → Tauri + Svelte

## Overview
Migrate from Flutter client to a modern Tauri + Svelte desktop application while keeping the existing Python server and Flutter client in their respective folders.

## Project Structure
```
wingbot/
├── wingbot_client/          # Existing Flutter client (kept for reference)
├── wingbot_server/          # Existing Python server (kept)
├── src/                     # NEW: Svelte frontend
├── src-tauri/              # NEW: Rust backend
├── package.json
├── components.json          # shadcn/ui configuration
└── plan.md                  # This file
```

## Migration Steps

### Phase 1: Setup Tauri + Svelte Project ✅ COMPLETED
1. ✅ Create plan.md
2. ✅ Initialize SvelteKit project with modern tooling
   - ✅ SvelteKit (latest) - Full-stack framework with routing and API endpoints
   - ✅ TailwindCSS v4 - Utility-first CSS framework for rapid UI development
   - ✅ shadcn/ui - Modern component library with Svelte support
   - ✅ TypeScript - Type safety and better development experience
   - ✅ Vite - Fast build tool and development server
3. ✅ Configure latest versions:
   - ✅ Tauri v2 - Added and configured for desktop app
   - ✅ Svelte v5 - Latest Svelte with modern syntax
4. ✅ Set up project structure with modular components

### Phase 2: Backend Development 🔄 IN PROGRESS
1. ✅ Basic Tauri backend structure
2. ✅ Implement WingBot-specific backend modules:
   - ✅ Game state management (champion select, live game) - Basic structure with mock data
   - ✅ AI service integration with existing Python backend - HTTP client + health check
   - ✅ League client integration (LCU API) - Windows detection working, Unix/Linux TODO
   - ✅ Configuration management - Basic structure, many methods unused
3. ✅ Create Tauri commands for frontend-backend communication

### Phase 3: Frontend Development 🔄 IN PROGRESS
1. ✅ shadcn/ui components configured
2. ✅ Basic layout and navigation:
   - ✅ Navigation rail with collapsible sidebar
   - ✅ Dashboard page with status cards and quick actions
   - ✅ Modular component library (StatusCard, QuickActionCard, SectionCard)
3. 🔄 Migrate remaining Flutter widgets to Svelte components:
   - 🔄 Champion select screen (pick/ban recommendations) - Route exists, page TODO
   - 🔄 Game info cards (live game data) - Route exists, page TODO
   - 🔄 Abilities cards (skill recommendations) - Route exists, page TODO
   - 🔄 Player cards (team/enemy information) - Route exists, page TODO
   - 🔄 Settings cards (configuration) - Route exists, page TODO
4. ✅ Ensure modular and reusable component architecture

### Phase 4: API Integration 🔄 IN PROGRESS
1. ✅ Basic Tauri command structure
2. 🔄 AI service health check integration - Health check implemented but not showing in UI
3. 🔄 League client detection - Windows working, Unix/Linux TODO
4. 📋 Integrate with existing Python server endpoints for real data
5. 📋 Implement proper error handling and loading states

### Phase 5: Features & Polish 📋 PLANNED
1. 📋 Replace mock data with real AI-powered recommendations
2. 📋 Add modern desktop-specific features
3. 📋 Ensure responsive design
4. 📋 Add proper TypeScript types
5. 📋 Implement proper state management

## Current Progress - Detailed Status

### ✅ **What's COMPLETED:**
- **SvelteKit Project**: Modern full-stack framework with TailwindCSS v4
- **Tauri Integration**: Desktop app framework configured and building successfully
- **shadcn/ui Setup**: Component library ready for use
- **Backend Modules**: All core modules created with basic functionality
- **Frontend Layout**: Navigation rail with collapsible sidebar
- **Dashboard**: Status cards, quick actions, and system information display
- **Component Library**: Modular, reusable components (StatusCard, QuickActionCard, SectionCard)
- **Tauri Commands**: All necessary commands registered and working

### 🔄 **What's PARTIALLY IMPLEMENTED:**
- **Game Manager**: Structure complete but returns mock data for suggestions/recommendations
- **AI Service**: HTTP client working, health check implemented but not integrated into system info
- **LCU API**: Windows detection working, Unix/Linux detection TODO
- **Configuration**: Basic structure in place but many methods unused
- **Frontend Routes**: Navigation exists but some destination pages are missing

### ❌ **What's MOCKED/PLACEHOLDER:**
1. **Game Data**: Champion suggestions and ability recommendations return hardcoded mock data
2. **AI Service Status**: Shows "Checking..." instead of actual connection status
3. **Champion Data**: Empty HashMap in GameManager
4. **Missing Pages**: Champion select, live game, AI chat, settings routes exist but pages don't

### 🚨 **Current Blockers:**
1. **AI Service Health Check**: Not integrated into system info display
2. **Mock Data**: Frontend shows placeholder data instead of real information
3. **Missing Pages**: Navigation links to non-existent routes
4. **Error Handling**: No proper error states or loading indicators

## Next Immediate Steps

### 1. **Fix AI Service Status Display** (High Priority)
- Integrate health check into `get_system_info` Tauri command
- Update frontend to show real connection status instead of "Checking..."

### 2. **Create Missing Route Pages** (High Priority)
- `/champion-select` - Champion pick/ban interface
- `/live-game` - In-game assistance and data
- `/ai-chat` - AI conversation interface
- `/settings` - Configuration panel

### 3. **Replace Mock Data** (Medium Priority)
- Implement real champion data loading
- Connect to Python backend for actual AI recommendations
- Add real-time game state updates

### 4. **Complete LCU API** (Medium Priority)
- Add Unix/Linux support for cross-platform compatibility
- Improve error handling and connection management

### 5. **Add Error Handling** (Medium Priority)
- Proper error states for failed API calls
- Loading indicators for async operations
- User feedback for connection issues

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
- [x] shadcn/ui components configured
- [x] Core backend modules implemented (structure)
- [x] Basic frontend layout and navigation
- [x] Modular component architecture established
- [ ] Mock data replaced with real functionality
- [ ] All existing Flutter functionality is replicated
- [ ] UI is modern and responsive
- [ ] Integration with existing server works
- [ ] Code follows user preferences

## Template Benefits for WingBot
- **SvelteKit routing** - Perfect for multi-screen desktop app navigation
- **API routes** - Can build backend endpoints right in the app
- **Component testing** - Built-in testing capabilities
- **Local database** - Can add SQLite for storing game data and preferences
- **Modern tooling** - Latest versions of all dependencies
- **Desktop-ready** - Optimized for Tauri desktop app performance
- **shadcn/ui** - Professional component library for consistent UI

## Current Development Status
**Phase 2 (Backend)**: 85% Complete - Core structure done, needs real data integration
**Phase 3 (Frontend)**: 60% Complete - Layout done, needs missing pages and real data
**Phase 4 (API Integration)**: 40% Complete - Commands exist, needs real data flow
**Overall Progress**: ~65% Complete - Solid foundation, needs functionality implementation
