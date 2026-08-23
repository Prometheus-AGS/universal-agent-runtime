#!/usr/bin/env bash
set -euo pipefail

output_path=""

usage() {
  echo "usage: $0 --output <service-env-file>" >&2
}

while (($#)); do
  case "$1" in
    --output)
      output_path=${2-}
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$output_path" ]]; then
  usage
  exit 2
fi

resolve_value() {
  local canonical=$1
  shift
  local candidate value

  for candidate in "$canonical" "$@"; do
    value=${!candidate-}
    if [[ -n "$value" ]]; then
      if [[ "$value" == *$'\n'* || "$value" == *$'\r'* ]]; then
        echo "provider credential $canonical contains a line break" >&2
        return 2
      fi
      printf '%s' "$value"
      return 0
    fi
  done
  return 1
}

quote_dotenv() {
  local value=$1
  value=${value//\'/\'\\\'\'}
  printf "'%s'" "$value"
}

output_dir=$(dirname "$output_path")
mkdir -p "$output_dir"
temporary=$(mktemp "$output_dir/.uar-env.XXXXXX")
trap 'rm -f "$temporary"' EXIT
chmod 600 "$temporary"

if [[ -f "$output_path" ]]; then
  while IFS= read -r line || [[ -n "$line" ]]; do
    case "$line" in
      KIMI_API_KEY=*|KIMI_CODING_API_KEY=*|KIMI_CODING_KEY=*|MINIMAX_API_KEY=*|MINIMAX_KEY=*|DASHSCOPE_API_KEY=*|QWEN_API_KEY=*|QWEN_TOKEN_PLAN_API_KEY=*|MOONSHOT_API_KEY=*|ZAI_API_KEY=*)
        ;;
      *)
        printf '%s\n' "$line" >> "$temporary"
        ;;
    esac
  done < "$output_path"
fi

append_resolved() {
  local canonical=$1
  shift
  local status value
  if value=$(resolve_value "$canonical" "$@"); then
    {
      printf '%s=' "$canonical"
      quote_dotenv "$value"
      printf '\n'
    } >> "$temporary"
  else
    status=$?
    if [[ $status -eq 2 ]]; then
      return 1
    fi
  fi
}

append_resolved KIMI_API_KEY KIMI_CODING_API_KEY KIMI_CODING_KEY
append_resolved MINIMAX_API_KEY MINIMAX_KEY
append_resolved DASHSCOPE_API_KEY QWEN_API_KEY QWEN_TOKEN_PLAN_API_KEY
append_resolved MOONSHOT_API_KEY
append_resolved ZAI_API_KEY

mv -f "$temporary" "$output_path"
chmod 600 "$output_path"
trap - EXIT
