#!/bin/bash
# Start CSV Explorer Services

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "==================================="
echo "  CSV Explorer - Starting Services"
echo "==================================="
echo ""

# Kill any existing instances
echo "Cleaning up old processes..."
pkill -f csv-explorer-api 2>/dev/null || true
pkill -f csv-explorer-ui 2>/dev/null || true
sleep 1

# Start API Server
echo ""
echo "[1/2] Starting API Server..."
nohup ./target/release/csv-explorer-api start > /tmp/api.log 2>&1 &
API_PID=$!
echo "  API PID: $API_PID"

# Wait for API
echo "  Waiting for API..."
for i in $(seq 1 10); do
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "  ✓ API ready at http://localhost:8080"
        break
    fi
    sleep 1
done

# Start UI Server
echo ""
echo "[2/2] Starting Web UI..."
nohup bash "$SCRIPT_DIR/serve-ui.sh" > /tmp/ui.log 2>&1 &
UI_PID=$!
echo "  UI PID: $UI_PID"

# Wait for UI
echo "  Waiting for UI..."
for i in $(seq 1 5); do
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        echo "  ✓ UI ready at http://localhost:3000"
        break
    fi
    sleep 1
done

echo "==================================="
echo "  Services Running"
echo "==================================="
echo ""
echo "  ✓ Web UI:     http://localhost:3000"
echo "  ✓ API Server: http://localhost:8080"
echo "  ✓ Health:     http://localhost:8080/health"
echo "  ✓ Stats:      http://localhost:8080/api/v1/stats"
echo ""
echo "PIDs:"
echo "  API: $API_PID"
echo "  UI:  $UI_PID"
echo ""
echo "To stop all services:"
echo "  kill $API_PID $UI_PID"
echo ""
echo "Logs:"
echo "  API: tail -f /tmp/api.log"
echo "  UI:  tail -f /tmp/ui.log"
echo ""
echo "Press Ctrl+C to stop all services..."
echo ""

# Cleanup function
cleanup() {
    echo ""
    echo "Stopping services..."
    kill $API_PID 2>/dev/null || true
    kill $UI_PID 2>/dev/null || true
    echo "Done."
}

# Trap Ctrl+C
trap cleanup INT TERM

# Keep running
wait
