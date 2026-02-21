#!/usr/bin/env bash
# start.sh — Bootstrap script for the UAR local production stack
#
# Usage:
#   ./start.sh          Start all services in detached mode
#   ./start.sh stop     Stop and remove containers (preserves volumes)
#   ./start.sh down     Stop and remove containers AND volumes (full reset)
#   ./start.sh logs     Tail logs from all services
#   ./start.sh status   Show service health

set -euo pipefail

COMPOSE_FILE="docker-compose.prod.yml"
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_header() {
  echo ""
  echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${CYAN}  Universal Agent Runtime — Local Production Stack${NC}"
  echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
}

print_urls() {
  echo -e "${GREEN}✓ Stack is running!${NC}"
  echo ""
  echo -e "  ${CYAN}UAR Application${NC}   →  http://localhost:3000"
  echo -e "  ${CYAN}UAR API Docs${NC}      →  http://localhost:3000/api/uar"
  echo -e "  ${CYAN}Memory MCP${NC}        →  http://localhost:3000/mcp/memory"
  echo -e "  ${CYAN}UAR MCP${NC}           →  http://localhost:3000/mcp/uar"
  echo -e "  ${CYAN}DbGate${NC}            →  http://localhost:5050"
  echo -e "  ${CYAN}Surrealist${NC}        →  http://localhost:8080"
  echo ""
}

# ── Subcommands ──────────────────────────────────────────────────────────────

CMD="${1:-start}"

case "$CMD" in
  stop)
    print_header
    echo "Stopping services (volumes preserved)…"
    docker compose -f "$COMPOSE_FILE" stop
    echo -e "${GREEN}✓ Services stopped.${NC}"
    exit 0
    ;;

  down)
    print_header
    echo -e "${RED}WARNING: This will remove all containers AND volumes (data will be lost).${NC}"
    read -r -p "Continue? [y/N] " confirm
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
      docker compose -f "$COMPOSE_FILE" down -v
      echo -e "${GREEN}✓ Stack removed including volumes.${NC}"
    else
      echo "Aborted."
    fi
    exit 0
    ;;

  logs)
    docker compose -f "$COMPOSE_FILE" logs -f --tail=100
    exit 0
    ;;

  status)
    docker compose -f "$COMPOSE_FILE" ps
    exit 0
    ;;

  start|"")
    # fall through to main start logic
    ;;

  *)
    echo "Usage: $0 [start|stop|down|logs|status]"
    exit 1
    ;;
esac

# ── Main start flow ──────────────────────────────────────────────────────────

print_header

# Check Docker is available
if ! command -v docker &>/dev/null; then
  echo -e "${RED}Error: Docker is not installed or not in PATH.${NC}"
  echo "Install Docker Desktop: https://www.docker.com/products/docker-desktop"
  exit 1
fi

# Check Compose v2
if ! docker compose version &>/dev/null; then
  echo -e "${RED}Error: Docker Compose v2 is required (docker compose, not docker-compose).${NC}"
  exit 1
fi

# Check for .env file
if [[ ! -f ".env" ]]; then
  echo -e "${YELLOW}No .env file found. Creating from .env.example…${NC}"
  cp .env.example .env
  echo ""
  echo -e "${YELLOW}ACTION REQUIRED:${NC}"
  echo "  Edit .env and set your API keys and secrets before the stack starts."
  echo ""
  echo "  Minimum required:"
  echo "    LLM_API_KEY         — your OpenAI or compatible LLM API key"
  echo "    LLM_BASE_URL        — LLM API endpoint (default: https://api.openai.com)"
  echo "    LLM_MODEL           — model name (default: gpt-4o)"
  echo "    OPENAI_API_KEY      — for memory embeddings"
  echo "    UAR_SECURITY__JWT_SECRET — generate with: openssl rand -base64 64"
  echo ""
  read -r -p "Open .env in your editor now? [Y/n] " open_env
  if [[ ! "$open_env" =~ ^[Nn]$ ]]; then
    "${EDITOR:-nano}" .env
  fi
fi

echo "Pulling latest images…"
docker compose -f "$COMPOSE_FILE" pull --quiet 2>/dev/null || true

echo "Starting services…"
docker compose -f "$COMPOSE_FILE" up -d

echo ""
echo "Waiting for UAR to become healthy…"

MAX_WAIT=120
ELAPSED=0
INTERVAL=5

while [ "$ELAPSED" -lt "$MAX_WAIT" ]; do
  STATUS=$(docker inspect --format='{{.State.Health.Status}}' uar-app 2>/dev/null || echo "missing")
  if [[ "$STATUS" == "healthy" ]]; then
    echo ""
    print_urls
    exit 0
  fi
  if [[ "$STATUS" == "unhealthy" ]]; then
    echo ""
    echo -e "${RED}UAR failed its healthcheck. Check logs with: ./start.sh logs${NC}"
    docker compose -f "$COMPOSE_FILE" ps
    exit 1
  fi
  printf "."
  sleep "$INTERVAL"
  ELAPSED=$((ELAPSED + INTERVAL))
done

echo ""
echo -e "${YELLOW}UAR is still starting (healthcheck pending). Logs:${NC}"
docker compose -f "$COMPOSE_FILE" logs uar --tail=30
echo ""
echo "Run './start.sh status' to check service health."
echo "Run './start.sh logs' to tail all logs."
