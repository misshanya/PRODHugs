#!/usr/bin/env bash
# Render a Markdown CI report into:
#   - $GITHUB_STEP_SUMMARY  (the per-run summary tab on GitHub Actions)
#   - $RUNNER_TEMP/report.md (read back by post-comment.js for the PR comment)
#
# The voice belongs to our perpetually exasperated DevOps engineer. She is
# not happy about cleaning up after you again, but she has scripts and she
# will use them. No decorative emojis — text markers only. Status icons
# (status circles) are allowed because they're functional, not vibes.

set -euo pipefail

target="${1:?expected target: backend or frontend}"
report="${RUNNER_TEMP:-/tmp}/report.md"

label_for() {
    case "${1:-skipped}" in
        success) echo "OK" ;;
        failure) echo "FAIL" ;;
        skipped) echo "skip" ;;
        cancelled) echo "cancelled" ;;
        *) echo "$1" ;;
    esac
}

dot_for() {
    case "${1:-skipped}" in
        success) echo "🟢" ;;
        failure) echo "🔴" ;;
        skipped) echo "⚪" ;;
        cancelled) echo "🟡" ;;
        *) echo "⚫" ;;
    esac
}

# Pick a snarky-but-fair opener for the given verdict. Each list is shuffled
# per-run so the same PR doesn't get the identical line three pushes in a
# row.
opener_pass=(
    "Ладно, всё зелёное. Не привыкай — в следующий раз я буду строже."
    "Чисто. Я даже немного разочарована, мне было что сказать."
    "Прошло. Можешь идти мёрджить, я тут постою посмотрю."
    "Хорошо. Хоть один раз без моих нервов."
)

opener_fail=(
    "Ну вот опять. Сядь, я объясню."
    "Слушай. У нас с тобой третий раз один и тот же разговор."
    "Я не злюсь, я просто очень разочарована. Смотри."
    "Окей. Перед мёрджем — почини вот это."
    "Только не говори мне, что у тебя локально всё работало."
)

pick_one() {
    local arr_name="$1"
    eval "local arr=(\"\${${arr_name}[@]}\")"
    local len="${#arr[@]}"
    local idx=$(( ( ${RANDOM:-0} + $$ ) % len ))
    echo "${arr[$idx]}"
}

# Russian plural agreement for "проверка" — 1 проверка, 2 проверки, 5 проверок.
plural_check() {
    local n=$1
    local last_two=$((n % 100))
    local last_one=$((n % 10))
    if [ "$last_two" -ge 11 ] && [ "$last_two" -le 14 ]; then
        echo "проверок"
    elif [ "$last_one" -eq 1 ]; then
        echo "проверка"
    elif [ "$last_one" -ge 2 ] && [ "$last_one" -le 4 ]; then
        echo "проверки"
    else
        echo "проверок"
    fi
}

# Per-check fix hints — short, imperative, copy-pasteable.
hint_for() {
    case "$1" in
        gofmt) echo "Запусти у себя: \`gofmt -w .\` в \`backend/\` и закоммить." ;;
        vet) echo "\`go vet ./...\` падает — это не вкусовщина, это реальный баг." ;;
        build) echo "\`go build ./...\` не собирается. Прежде чем пушить — соберись локально." ;;
        test) echo "Тесты упали. \`go test ./...\` локально и почини." ;;
        lint) echo "\`golangci-lint run\` — посмотри список ниже, не паникуй, там всё конкретно." ;;
        oapi_v1) echo "Перегенерируй v1: \`cd backend && oapi-codegen -config oapi-codegen.yml api/openapi.yaml\` и закоммить \`internal/transport/http/v1/api.gen.go\`." ;;
        oapi_v2) echo "Перегенерируй v2: \`cd backend && oapi-codegen -config oapi-codegen-v2.yml api/openapi-v2.yaml\` и закоммить \`internal/transport/http/v2/api.gen.go\`." ;;
        sqlc) echo "\`cd backend && sqlc generate -f internal/db/sqlc/sqlc.yaml\` и закоммить \`internal/db/sqlc/storage/\`." ;;
        docker) echo "Образ не собрался. Скорее всего сломалось то же, что и обычный build/lint выше — почини их, и Docker подтянется." ;;
        oxlint) echo "\`bun run lint:oxlint\` локально, прочитай что пишет — обычно лечится за полминуты." ;;
        eslint) echo "\`bun run lint:eslint --fix\` сначала, потом разберись с тем, что осталось." ;;
        typecheck) echo "\`bun run type-check\` — типы реально не сходятся, не игнорируй." ;;
        *) echo "" ;;
    esac
}

case "$target" in
    backend)
        title="Backend"
        checks=(
            "gofmt|R_GOFMT|gofmt.out|gofmt"
            "go vet|R_VET|vet.out|vet"
            "go build|R_BUILD|build.out|build"
            "go test|R_TEST|test.out|test"
            "golangci-lint|R_LINT|lint.out|lint"
            "OpenAPI v1 codegen drift|R_OAPI_V1|oapi_v1.out|oapi_v1"
            "OpenAPI v2 codegen drift|R_OAPI_V2|oapi_v2.out|oapi_v2"
            "sqlc codegen drift|R_SQLC|sqlc.out|sqlc"
            "Docker image build|R_DOCKER||docker"
        )
        ;;
    frontend)
        title="Frontend"
        checks=(
            "oxlint|R_OXLINT|oxlint.out|oxlint"
            "eslint|R_ESLINT|eslint.out|eslint"
            "type-check|R_TYPECHECK|typecheck.out|typecheck"
            "build|R_BUILD|build.out|build"
            "Docker image build|R_DOCKER||docker"
        )
        ;;
    *)
        echo "unknown target: $target" >&2
        exit 2
        ;;
esac

# Overall verdict.
overall="success"
failed_count=0
for c in "${checks[@]}"; do
    IFS='|' read -r _ var _ _ <<< "$c"
    val="${!var:-skipped}"
    if [ "$val" = "failure" ]; then
        overall="failure"
        failed_count=$((failed_count + 1))
    fi
done

# Build the report.
{
    echo "## CI — $title"
    echo
    if [ "$overall" = "success" ]; then
        echo "> $(pick_one opener_pass)"
    else
        echo "> $(pick_one opener_fail)"
    fi
    echo
    if [ "$overall" = "success" ]; then
        echo "**Итог:** всё зелёное."
    else
        echo "**Итог:** упало $failed_count $(plural_check "$failed_count"). Детали — ниже."
    fi
    echo
    echo "Коммит: \`${GITHUB_SHA:0:7}\` · [полный лог запуска](${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID})"
    echo
    echo "| | Проверка | Статус |"
    echo "| --- | --- | --- |"
    for c in "${checks[@]}"; do
        IFS='|' read -r label var _ _ <<< "$c"
        val="${!var:-skipped}"
        echo "| $(dot_for "$val") | $label | $(label_for "$val") |"
    done
    echo

    # Detail blocks for every failure.
    any_failed=false
    for c in "${checks[@]}"; do
        IFS='|' read -r label var file id <<< "$c"
        val="${!var:-skipped}"
        [ "$val" = "failure" ] || continue
        any_failed=true
        echo "### $label — упало"
        echo
        hint="$(hint_for "$id")"
        if [ -n "$hint" ]; then
            echo "$hint"
            echo
        fi
        if [ -n "$file" ] && [ -f "${RUNNER_TEMP:-/tmp}/$file" ]; then
            echo "<details><summary>что именно сказал инструмент</summary>"
            echo
            echo '```'
            # Cap at ~12 KB / 200 lines so we stay well under the comment
            # length limit (65 KB) and the run summary limit (1 MB) even
            # with several failing steps.
            head -c 12288 "${RUNNER_TEMP:-/tmp}/$file" | head -n 200
            echo
            echo '```'
            echo
            echo "</details>"
            echo
        fi
    done

    if [ "$any_failed" = false ]; then
        echo "_Больше тут ничего нет — всё прошло._"
        echo
    fi

    echo "---"
    echo "<sub>Этот комментарий обновляется на каждый пуш в этот PR. Сообщение от вашей вечно недовольной CI-инженерки.</sub>"
} > "$report"

# Mirror into the run summary tab so it's also visible without opening the PR.
cat "$report" >> "${GITHUB_STEP_SUMMARY:-/dev/stdout}"
