# CSV Explorer - Web UI Working! ✅

## Quick Start

```bash
./start.sh
```

Then open: **http://localhost:3000**

## What You'll See

### Home Dashboard (`/`)
- **Statistics Cards**:
  - Total Rights: 8
  - Total Transfers: 7
  - Active Seals: 11
  - Contracts: 11

- **Chain Status**: Real-time status for all 5 chains
  - Bitcoin, Ethereum, Sui, Aptos, Solana
  - Green indicators for synced chains

### Rights Page (`/rights`)
Table showing all 8 rights with:
- Right ID (monospace, blue)
- Chain name
- Owner address
- Status badge (active/spent/pending)
- Transfer count

### Transfers Page (`/transfers`)
Table showing all 7 transfers with:
- Transfer ID
- Route (from_chain → to_chain)
- Status badge (completed/in_progress/pending/failed)
- Creation date

### Seals Page (`/seals`)
Table showing all 11 seals with:
- Seal ID
- Chain
- Seal type (UTXO, Object, Resource, Nullifier, Account)
- Status (available/consumed)
- Block height

### Statistics Page (`/stats`)
Detailed breakdown:
- Transfer success rate: 80.0%
- Average transfer time: 4.0 hours
- Rights by chain distribution
- Transfers by chain pair
- Active seals by chain

## How It Works

The Web UI is a **single-page application** built with:
- **HTML5** - Structure
- **Tailwind CSS** - Styling (via CDN)
- **Vanilla JavaScript** - API integration
- **No WASM required** - Runs directly in browser

It connects to the API server at `http://localhost:8080` and fetches data dynamically.

## Architecture

```
┌─────────────────┐
│     Browser     │
│  http://:3000   │
│  (Static HTML)  │
└────────┬────────┘
         │
    Fetch API
         │
         v
┌─────────────────┐
│  API Server     │
│  http://:8080   │
│  (Rust/Axum)    │
└────────┬────────┘
         │
    SQL Queries
         │
         v
┌─────────────────┐
│  SQLite DB      │
│  explorer.db    │
└─────────────────┘
```

## Files

```
ui/web/
└── index.html          # Single-page web UI

serve-ui.sh             # Serves UI using Python HTTP server
start.sh                # Starts both API and UI
```

## API Endpoints Used

The UI calls these API endpoints:

```javascript
GET /health                     // Health check
GET /api/v1/stats               // Aggregate statistics
GET /api/v1/rights?limit=50     // List rights
GET /api/v1/transfers?limit=50  // List transfers
GET /api/v1/seals?limit=50      // List seals
```

## Customization

### Change Port
```bash
UI_PORT=8000 ./start.sh
```

### Change API URL
Edit `ui/web/index.html`:
```javascript
const API_URL = 'http://your-api-server:8080';
```

### Styling
Edit the `<style>` block in `ui/web/index.html`:
```css
body { background-color: #030712; }
.card { background-color: #111827; }
```

## Features

✅ **Responsive Design** - Works on mobile and desktop
✅ **Dark Theme** - Matches terminal aesthetic
✅ **Live Data** - Fetches from API in real-time
✅ **Status Badges** - Color-coded status indicators
✅ **Navigation** - Single-page app with client-side routing
✅ **Auto-refresh** - Health check every 30 seconds

## Troubleshooting

### UI Not Loading
```bash
# Check if UI server is running
curl http://localhost:3000

# Restart UI
pkill -f "http.server"
bash serve-ui.sh
```

### No Data Showing
```bash
# Check API is running
curl http://localhost:8080/health

# Check API returns data
curl http://localhost:8080/api/v1/stats
```

### CORS Issues
The UI and API are on different ports. If you see CORS errors:

1. The API already has CORS enabled
2. Make sure both are running on localhost
3. Check browser console for errors

## Next Steps

1. **Browse the Data**: Click through different pages
2. **Explore Details**: View individual rights, transfers, seals
3. **Monitor Stats**: Check the statistics page
4. **Customize**: Modify the HTML/CSS to match your preferences

## Current Status

✅ **API Server**: Running on port 8080
✅ **Web UI**: Running on port 3000
✅ **Database**: Seeded with test data
✅ **All Features**: Working and tested

**Open http://localhost:3000 in your browser!**
