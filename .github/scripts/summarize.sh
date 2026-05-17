#!/usr/bin/env bash
# Render a Markdown report of CI step results into $GITHUB_STEP_SUMMARY.
#
# Called from .github/workflows/pr-checks.yml as:
#   summarize.sh <backend|frontend>
#
# The workflow exports one R_<STEP_ID> env var per check (value is the GitHub
# Actions outcome — "success" / "failure" / "skipped" / "cancelled") and pipes
# each step's output to $RUNNER_TEMP/<id>.out. We turn that into a table plus
# an expandable block of output for any failing step.

set -euo pipefail

target="${1:?expected target: backend or frontend}"
out="${GITHUB_STEP_SUMMARY:-/dev/stdout}"

emoji() {
    case "${1:-skipped}" in
        success) echo "✅" ;;
        failure) echo "❌" ;;
        skipped) echo "⏭️" ;;
        cancelled) echo "🚫" ;;
        *) echo "❓" ;;
    esac
}

human() {
    case "${1:-skipped}" in
        success) echo "passed" ;;
        failure) echo "failed" ;;
        skipped) echo "skipped" ;;
        cancelled) echo "cancelled" ;;
        *) echo "$1" ;;
    esac
}

# Each check is "label|env_var|out_file|fix_hint".
case "$target" in
    backend)
        title="🔧 Backend checks"
        checks=(
            "gofmt|R_GOFMT|gofmt.out|gofmt -w ."
            "go vet|R_VET|vet.out|"
            "go build|R_BUILD|build.out|"
            "go test|R_TEST|test.out|"
            "golangci-lint|R_LINT||golangci-lint run"
            "OpenAPI v1 codegen drift|R_OAPI_V1|oapi_v1.out|oapi-codegen -config oapi-codegen.yml api/openapi.yaml"
            "OpenAPI v2 codegen drift|R_OAPI_V2|oapi_v2.out|oapi-codegen -config oapi-codegen-v2.yml api/openapi-v2.yaml"
            "sqlc codegen drift|R_SQLC|sqlc.out|sqlc generate -f internal/db/sqlc/sqlc.yaml"
            "Docker image build|R_DOCKER||"
        )
        ;;
    frontend)
        title="🎨 Frontend checks"
        checks=(
            "Lint (oxlint)|R_OXLINT|oxlint.out|bun run lint:oxlint"
            "Lint (eslint)|R_ESLINT|eslint.out|bun run lint:eslint"
            "Type-check|R_TYPECHECK|typecheck.out|bun run type-check"
            "Build|R_BUILD|build.out|bun run build-only"
            "Docker image build|R_DOCKER||"
        )
        ;;
    *)
        echo "unknown target: $target" >&2
        exit 2
        ;;
esac

# Overall verdict — start the summary with a one-line conclusion so the PR
# preview shows it without expanding.
overall="success"
for c in "${checks[@]}"; do
    IFS='|' read -r _ var _ _ <<< "$c"
    val="${!var:-skipped}"
    if [ "$val" = "failure" ]; then overall="failure"; fi
done

{
    if [ "$overall" = "success" ]; then
        echo "## $title — $(emoji success) all green"
    else
        echo "## $title — $(emoji failure) failures detected"
    fi
    echo
    echo "Commit: \`${GITHUB_SHA:0:7}\` · Workflow run: [#${GITHUB_RUN_NUMBER}](${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID})"
    echo
    echo "| | Check | Result |"
    echo "| --- | --- | --- |"
    for c in "${checks[@]}"; do
        IFS='|' read -r label var _ _ <<< "$c"
        val="${!var:-skipped}"
        echo "| $(emoji "$val") | $label | $(human "$val") |"
    done
    echo
} >> "$out"

# For every failing check, emit a collapsible block with its captured stdout
# and (when known) a copy-pasteable fix hint.
any_failed=false
for c in "${checks[@]}"; do
    IFS='|' read -r label var file fix <<< "$c"
    val="${!var:-skipped}"
    [ "$val" = "failure" ] || continue
    any_failed=true
    {
        echo "### ❌ $label"
        if [ -n "$fix" ]; then
            echo
            echo "Likely fix:"
            echo
            echo '```sh'
            echo "$fix"
            echo '```'
        fi
        if [ -n "$file" ] && [ -f "${RUNNER_TEMP:-/tmp}/$file" ]; then
            echo
            echo "<details><summary>Captured output</summary>"
            echo
            echo '```'
            # Cap each block at 200 lines / 16 KB so we don't blow past
            # GitHub's 1 MB summary limit on a runaway log.
            head -c 16384 "${RUNNER_TEMP:-/tmp}/$file" | head -n 200
            echo
            echo '```'
            echo
            echo "</details>"
        fi
        echo
    } >> "$out"
done

if [ "$any_failed" = false ]; then
    {
        echo
        echo "> Nothing more to report — every check is green. 🎉"
    } >> "$out"
fi
