#!/usr/bin/env bash
#
# Smoke-тест собранного образа webaggregator.
#
# В отличие от kcs-monitor, это приложение не стартует без БД: main() читает
# DATABASE_URL и сразу подключается к Postgres. Поэтому тест ожидает живой
# Postgres на 127.0.0.1:5432 (в CI его поднимает секция services) и проверяет,
# что контейнер отвечает на главной странице и отдаёт статику, которую читает
# с диска (StaticService: ./static/style.css).
#
# Дополнительно (SMOKE_SEED=1, по умолчанию) тест создаёт таблицу candles по
# схеме из миграций kcs-monitor, кладёт в неё пару свечей и дёргает JSON-ручки
# /api/candles*: так проверяется не только запуск образа, но и SQL новых ручек
# на настоящем Postgres.
#
# Контейнер запускается с --network host — так же, как в docker-compose.yml.
#
# Переменные: IMAGE_ID (id или тег образа), DATABASE_URL, SMOKE_PORT (8080),
#             SMOKE_SEED (1/0), SMOKE_PSQL_IMAGE (postgres:18-alpine).

set -euo pipefail

image="${IMAGE_ID:-${IMAGE:-}}"
if [[ -z "$image" ]]; then
  echo "ОШИБКА: не задан IMAGE_ID (id или тег собранного образа)" >&2
  exit 1
fi

: "${DATABASE_URL:?DATABASE_URL не задан}"

port="${SMOKE_PORT:-8080}"
container="${SMOKE_CONTAINER:-webaggregator-smoke}"
base="http://127.0.0.1:${port}"
attempts="${SMOKE_ATTEMPTS:-20}"
seed="${SMOKE_SEED:-1}"
psql_image="${SMOKE_PSQL_IMAGE:-postgres:18-alpine}"
tmp_body="$(mktemp)"

# Схема — копия миграций kcs-monitor (источник истины там); свечи кладём
# только для своей тестовой биржи, чтобы не мешать реальным данным.
seed_sql=$(
  cat <<'SQL'
CREATE TABLE IF NOT EXISTS candles (
    symbol      text        NOT NULL,
    timeframe   text        NOT NULL,
    start_ts    bigint      NOT NULL,
    open        numeric     NOT NULL,
    high        numeric     NOT NULL,
    low         numeric     NOT NULL,
    close       numeric     NOT NULL,
    volume      numeric     NOT NULL,
    turnover    numeric     NOT NULL DEFAULT 0,
    update_time timestamptz NOT NULL DEFAULT now(),
    exchange    text        NOT NULL,
    PRIMARY KEY (exchange, symbol, timeframe, start_ts)
);
INSERT INTO candles
    (exchange, symbol, timeframe, start_ts, open, high, low, close, volume, turnover)
VALUES
    ('smoketest', 'SMOKE-USDT', '1hour', 1788998400, 100.5, 102.25, 99.75, 101.5, 12.5, 1268.75),
    ('smoketest', 'SMOKE-USDT', '1hour', 1789002000, 101.5, 103.75, 100.25, 102.5, 13.5, 1383.75)
ON CONFLICT (exchange, symbol, timeframe, start_ts) DO NOTHING;
SQL
)

cleanup() {
  rm -f "$tmp_body"
  docker rm -f "$container" >/dev/null 2>&1 || true
}
trap cleanup EXIT

if [[ "$seed" == "1" ]]; then
  echo "Готовлю таблицу candles через образ $psql_image"
  docker run --rm --network host "$psql_image" \
    psql "$DATABASE_URL" -v ON_ERROR_STOP=1 -q -c "$seed_sql"
fi

echo "Запускаю контейнер '$container' из образа '$image' (порт $port)"

# Имя контейнера фиксировано — на всякий случай убираем возможный остаток.
docker rm -f "$container" >/dev/null 2>&1 || true

docker run -d --name "$container" --network host \
  -e "DATABASE_URL=$DATABASE_URL" \
  -e "RUST_LOG=info" \
  "$image" >/dev/null

# Ждёт, что путь отдаёт ожидаемый код ответа (приложение поднимается не мгновенно).
wait_for_status() {
  local path="$1" expected="${2:-200}" code=""

  for _ in $(seq 1 "$attempts"); do
    code="$(curl -s -o /dev/null -w '%{http_code}' "${base}${path}" || true)"
    if [[ "$code" == "$expected" ]]; then
      echo "OK: GET $path -> $code"
      return 0
    fi
    sleep 1
  done

  echo "ОШИБКА: GET $path вернул '$code', ожидался $expected" >&2
  return 1
}

# Ждёт, что тело ответа содержит нужный фрагмент (проверка данных из БД).
wait_for_body() {
  local path="$1" needle="$2" code="" body=""

  for _ in $(seq 1 "$attempts"); do
    code="$(curl -sS -o "$tmp_body" -w '%{http_code}' "${base}${path}" || true)"
    body="$(cat "$tmp_body")"
    if [[ "$code" == "200" && "$body" == *"$needle"* ]]; then
      echo "OK: GET $path содержит '$needle'"
      return 0
    fi
    sleep 1
  done

  echo "ОШИБКА: GET $path (код $code) не содержит '$needle'" >&2
  echo "ответ: ${body:0:500}" >&2
  return 1
}

status=0
# Главная страница — приложение поднялось и отвечает.
wait_for_status / 200 || status=1
# Статика — файлы из образа доступны процессу под nonroot (важно для distroless).
wait_for_status /static/style.css 200 || status=1

# JSON-ручки: справочник серий, выборка с фильтрами и серия для графика.
if [[ "$seed" == "1" ]]; then
  wait_for_body '/api/candles/meta?exchange=smoketest' 'SMOKE-USDT' || status=1
  wait_for_body '/api/candles?exchange=smoketest&symbol=SMOKE-USDT&timeframe=1hour&order=asc' '"close":101.5' || status=1
  wait_for_body '/api/candles/latest?exchange=smoketest&timeframe=1hour' '1789002000' || status=1
  wait_for_body '/api/candles/series/SMOKE-USDT?timeframe=1hour' '"turnover":1383.75' || status=1
fi

if (( status != 0 )); then
  echo '--- логи контейнера ---' >&2
  docker logs "$container" >&2 || true
  exit 1
fi

echo 'Smoke-тест пройден'
