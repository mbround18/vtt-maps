#!/bin/bash

# Check rebuild status script for VTT Maps container

echo "=== VTT Maps Rebuild Status ==="
echo

# Check container uptime
echo "📊 Container Information:"
if [ -f /proc/1/stat ]; then
    # Get container start time in jiffies
    START_TIME=$(awk '{print $22}' /proc/1/stat 2>/dev/null)
    if [ -n "$START_TIME" ]; then
        # Convert jiffies to seconds (assuming 100 Hz)
        UPTIME_SECONDS=$((START_TIME / 100))
        UPTIME_MINUTES=$((UPTIME_SECONDS / 60))
        UPTIME_HOURS=$((UPTIME_MINUTES / 60))
        echo "   Container uptime: ${UPTIME_HOURS}h ${UPTIME_MINUTES}m ${UPTIME_SECONDS}s"
    else
        echo "   Container uptime: Unable to determine"
    fi
else
    echo "   Container uptime: /proc/1/stat not available"
fi

# Check environment
if [ "$CONTAINER" = "true" ]; then
    echo "   Environment: Container (CONTAINER=true)"
else
    echo "   Environment: Unknown"
fi

echo

# Check for lock file
LOCK_FILE="/data/assets/thumbnails/.map_rebuild_lock.json"
echo "🔍 Lock File Status:"
if [ -f "$LOCK_FILE" ]; then
    echo "   Lock file exists: $LOCK_FILE"
    echo "   Lock file contents:"
    cat "$LOCK_FILE" | jq . 2>/dev/null || cat "$LOCK_FILE"
    echo
    
    # Check if it's a stale lock (in container environment)
    if [ "$CONTAINER" = "true" ]; then
        echo "   🐳 Running in container - lock may be stale from previous run"
    fi
else
    echo "   No lock file found - rebuild is idle"
fi

echo

# Check API status if server is running
echo "🌐 API Status Check:"
API_URL="${API_URL:-http://localhost:8080}"

# Try to get rebuild status from API
if command -v curl >/dev/null 2>&1; then
    echo "   Checking rebuild status at $API_URL/api/maps/rebuild..."
    curl -s "$API_URL/api/maps/rebuild" | jq . 2>/dev/null || echo "   API not responding or invalid JSON"
    echo
else
    echo "   curl not available - cannot check API status"
fi

echo

# Provide helpful commands
echo "💡 Helpful Commands:"
echo "   Clear stale lock: rm -f '$LOCK_FILE'"
echo "   Check server logs: docker logs <container-name>"
echo "   Check disk space: df -h /data"
echo "   Check memory usage: free -h"

# Check available space
echo
echo "💾 Storage Information:"
df -h /data 2>/dev/null || echo "   /data not available"
