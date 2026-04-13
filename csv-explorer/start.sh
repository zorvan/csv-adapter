#!/bin/bash
# Start CSV Explorer services

set -e

EXPLORER_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PID_FILE="$EXPLORER_DIR/.pids"

# Create PID file if it doesn't exist
touch "$PID_FILE"

start_api() {
    echo "🚀 Starting CSV Explorer API server..."
    if pgrep -f "csv-explorer-api" > /dev/null 2>&1; then
        echo "✅ API server is already running"
    else
        cd "$EXPLORER_DIR"
        nohup cargo run -p csv-explorer-api -- start > /tmp/csv-explorer-api.log 2>&1 &
        echo $! >> "$PID_FILE"
        sleep 3
        
        # Health check
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            echo "✅ API server started successfully (port 8080)"
        else
            echo "❌ API server failed to start. Check logs: /tmp/csv-explorer-api.log"
            exit 1
        fi
    fi
}

start_ui() {
    echo "🎨 Starting CSV Explorer UI..."
    if pgrep -f "csv-explorer-ui" > /dev/null 2>&1; then
        echo "✅ UI server is already running"
    else
        cd "$EXPLORER_DIR"
        nohup cargo run -p csv-explorer-ui -- serve > /tmp/csv-explorer-ui.log 2>&1 &
        echo $! >> "$PID_FILE"
        sleep 3
        echo "✅ UI server started (port 3000)"
    fi
}

stop_all() {
    echo "🛑 Stopping all CSV Explorer services..."
    pkill -f "csv-explorer-api" 2>/dev/null || true
    pkill -f "csv-explorer-ui" 2>/dev/null || true
    rm -f "$PID_FILE"
    echo "✅ All services stopped"
}

status() {
    echo "📊 CSV Explorer Status:"
    if curl -s http://localhost:8080/health > /dev/null 2>&1; then
        echo "  ✅ API Server (8080): Running"
    else
        echo "  ❌ API Server (8080): Not running"
    fi
    
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        echo "  ✅ UI Server (3000): Running"
    else
        echo "  ❌ UI Server (3000): Not running"
    fi
}

case "${1:-start}" in
    start)
        start_api
        start_ui
        echo ""
        echo "🌐 Access the explorer at: http://localhost:3000"
        echo "📊 API at: http://localhost:8080"
        echo "🔍 GraphQL Playground: http://localhost:8080/playground"
        ;;
    stop)
        stop_all
        ;;
    restart)
        stop_all
        sleep 2
        start_api
        start_ui
        ;;
    status)
        status
        ;;
    *)
        echo "Usage: $0 {start|stop|restart|status}"
        exit 1
        ;;
esac
