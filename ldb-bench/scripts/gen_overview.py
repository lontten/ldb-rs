#!/usr/bin/env python3
"""从 Criterion 输出生成可读性总览页 target/criterion/index.html。"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

KNOWN_DBS = ("mysql", "postgres")
KNOWN_OPS = ("insert", "update", "delete", "first", "list", "count")
OP_DESC = {
    "insert": "插入 n 行",
    "update": "按条件更新",
    "delete": "按条件删除",
    "first": "取首行",
    "list": "列表查询",
    "count": "计数",
}
SKIP_DIRS = {"report", "base", "change", "new"}

CSS = """
    :root {
      --bg: #f7f6f2;
      --surface: #ffffff;
      --text: #1c1917;
      --muted: #57534e;
      --border: #e7e5e4;
      --accent: #0f766e;
      --bar: #0d9488;
      --bar-fast: #0f766e;
      --badge-bg: #ecfdf5;
      --badge-text: #065f46;
      --fast-row: #f0fdf4;
    }

    * { box-sizing: border-box; }

    body {
      margin: 0;
      font-family: "IBM Plex Sans", "Noto Sans SC", "PingFang SC", "Hiragino Sans GB",
        "Microsoft YaHei", sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.5;
    }

    .wrap {
      max-width: 920px;
      margin: 0 auto;
      padding: 2rem 1.25rem 3rem;
    }

    header {
      display: flex;
      flex-wrap: wrap;
      align-items: flex-start;
      justify-content: space-between;
      gap: 0.75rem 1.5rem;
      margin-bottom: 1.25rem;
    }

    h1 {
      margin: 0 0 0.35rem;
      font-size: 1.65rem;
      font-weight: 650;
      letter-spacing: -0.02em;
    }

    .tagline {
      margin: 0;
      color: var(--muted);
      font-size: 0.95rem;
    }

    .badge {
      display: inline-block;
      padding: 0.3rem 0.65rem;
      border-radius: 4px;
      background: var(--badge-bg);
      color: var(--badge-text);
      font-size: 0.8rem;
      font-weight: 600;
      white-space: nowrap;
    }

    .howto {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 1rem 1.15rem;
      margin-bottom: 1.5rem;
    }

    .howto h2 {
      margin: 0 0 0.5rem;
      font-size: 0.95rem;
      font-weight: 650;
    }

    .howto ol {
      margin: 0;
      padding-left: 1.2rem;
      color: var(--muted);
      font-size: 0.9rem;
    }

    .howto li { margin: 0.25rem 0; }

    .toolbar {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 0.75rem 1.25rem;
      margin-bottom: 1.25rem;
    }

    .toolbar-group {
      display: flex;
      flex-wrap: wrap;
      align-items: center;
      gap: 0.5rem;
    }

    .toolbar-label {
      font-size: 0.85rem;
      color: var(--muted);
    }

    .tabs {
      display: inline-flex;
      border: 1px solid var(--border);
      border-radius: 6px;
      overflow: hidden;
      background: var(--surface);
    }

    .tabs button {
      border: 0;
      background: transparent;
      padding: 0.45rem 1rem;
      font: inherit;
      font-size: 0.9rem;
      color: var(--muted);
      cursor: pointer;
    }

    .tabs button[aria-selected="true"] {
      background: var(--accent);
      color: #fff;
      font-weight: 600;
    }

    .tabs button:not([aria-selected="true"]):hover {
      background: #f5f5f4;
      color: var(--text);
    }

    .empty {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 1.25rem;
      color: var(--muted);
      font-size: 0.95rem;
    }

    .panel[hidden] { display: none; }

    .op {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 6px;
      margin-bottom: 1rem;
      overflow: hidden;
    }

    .op-head {
      padding: 0.7rem 1rem;
      border-bottom: 1px solid var(--border);
      font-weight: 650;
      font-size: 0.95rem;
      display: flex;
      align-items: baseline;
      gap: 0.5rem;
    }

    .op-head .op-label {
      font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
      color: var(--accent);
    }

    .op-head .op-desc {
      font-weight: 400;
      color: var(--muted);
      font-size: 0.85rem;
    }

    table {
      width: 100%;
      border-collapse: collapse;
      font-size: 0.9rem;
    }

    th, td {
      padding: 0.55rem 1rem;
      text-align: left;
      vertical-align: middle;
    }

    th {
      font-size: 0.75rem;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.04em;
      color: var(--muted);
      border-bottom: 1px solid var(--border);
      background: #fafaf9;
    }

    td { border-bottom: 1px solid var(--border); }

    tr:last-child td { border-bottom: 0; }

    tr.fastest {
      background: var(--fast-row);
    }

    .orm {
      font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
      font-weight: 600;
    }

    .ms {
      font-variant-numeric: tabular-nums;
      white-space: nowrap;
    }

    .rel {
      font-variant-numeric: tabular-nums;
      white-space: nowrap;
    }

    .rel.best {
      color: var(--accent);
      font-weight: 650;
    }

    .bar-cell { width: 42%; min-width: 120px; }

    .bar-track {
      height: 8px;
      background: #e7e5e4;
      border-radius: 4px;
      overflow: hidden;
    }

    .bar-fill {
      height: 100%;
      border-radius: 4px;
      background: var(--bar);
    }

    tr.fastest .bar-fill { background: var(--bar-fast); }

    footer {
      margin-top: 2rem;
      padding-top: 1rem;
      border-top: 1px solid var(--border);
      color: var(--muted);
      font-size: 0.85rem;
    }

    footer a {
      color: var(--accent);
      text-decoration: underline;
      text-underline-offset: 2px;
    }

    @media (max-width: 640px) {
      th:nth-child(4), td:nth-child(4) { display: none; }
      .bar-cell { width: auto; }
    }
"""


def parse_group_dir(name: str) -> tuple[str, str] | None:
    """mysql_count → (mysql, count)。"""
    for db in KNOWN_DBS:
        prefix = db + "_"
        if name.startswith(prefix):
            op = name[len(prefix) :]
            if op in KNOWN_OPS:
                return db, op
    return None


def read_mean_ms(estimates_path: Path) -> float | None:
    try:
        data = json.loads(estimates_path.read_text(encoding="utf-8"))
        ns = data["mean"]["point_estimate"]
        return float(ns) / 1_000_000.0
    except (OSError, KeyError, TypeError, ValueError, json.JSONDecodeError):
        return None


def collect(criterion_root: Path) -> dict:
    """DATA[db][n_str][op][orm] = ms"""
    data: dict = defaultdict(lambda: defaultdict(lambda: defaultdict(dict)))

    if not criterion_root.is_dir():
        return {}

    for group_dir in sorted(criterion_root.iterdir()):
        if not group_dir.is_dir() or group_dir.name in SKIP_DIRS:
            continue
        parsed = parse_group_dir(group_dir.name)
        if parsed is None:
            continue
        db, op = parsed

        for orm_dir in sorted(group_dir.iterdir()):
            if not orm_dir.is_dir() or orm_dir.name in SKIP_DIRS:
                continue
            orm = orm_dir.name
            for n_dir in sorted(orm_dir.iterdir()):
                if not n_dir.is_dir() or not n_dir.name.isdigit():
                    continue
                estimates = n_dir / "new" / "estimates.json"
                if not estimates.is_file():
                    continue
                ms = read_mean_ms(estimates)
                if ms is None:
                    continue
                data[db][n_dir.name][op][orm] = round(ms, 3)

    # 转为普通 dict 便于 JSON
    out: dict = {}
    for db, sizes in data.items():
        out[db] = {}
        for n, ops in sizes.items():
            out[db][n] = {op: dict(orms) for op, orms in ops.items()}
    return out


def render_html(data: dict) -> str:
    data_json = json.dumps(data, ensure_ascii=False, separators=(",", ":"))
    ops_json = json.dumps(
        [{"id": op, "title": op, "desc": OP_DESC[op]} for op in KNOWN_OPS],
        ensure_ascii=False,
        separators=(",", ":"),
    )
    dbs = [db for db in KNOWN_DBS if db in data]
    sizes: set[str] = set()
    for db in dbs:
        sizes.update(data[db].keys())
    size_list = sorted(sizes, key=lambda s: int(s))
    default_db = dbs[0] if dbs else "mysql"
    default_n = "50" if "50" in sizes else (size_list[-1] if size_list else "50")

    db_buttons = []
    for i, db in enumerate(KNOWN_DBS):
        label = "MySQL" if db == "mysql" else "PostgreSQL"
        selected = "true" if db == default_db else "false"
        disabled = "" if db in data else " disabled"
        db_buttons.append(
            f'<button type="button" role="tab" data-db="{db}" '
            f'aria-selected="{selected}"{disabled}>{label}</button>'
        )

    n_buttons = []
    for n in size_list or ["10", "50"]:
        selected = "true" if n == default_n else "false"
        n_buttons.append(
            f'<button type="button" role="tab" data-n="{n}" '
            f'aria-selected="{selected}">n = {n}</button>'
        )

    return f"""<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>ldb CRUD 性能对比</title>
  <style>{CSS}
  </style>
</head>
<body>
  <div class="wrap">
    <header>
      <div>
        <h1>ldb CRUD 性能对比</h1>
        <p class="tagline">数字越小越快。同库、同操作、同数据规模下比较各 ORM。</p>
      </div>
      <span class="badge">CI 生成</span>
    </header>

    <section class="howto" aria-label="怎么读这张表">
      <h2>怎么读</h2>
      <ol>
        <li>测的是一次完整 CRUD 迭代的平均耗时（毫秒），含建连与表重置/灌数准备。</li>
        <li>用上方切换数据库与数据规模 <strong>n</strong>（表中相关行数）。</li>
        <li>只在同一数据库、同一操作、同一 n 内横向比较；不要跨操作或跨库直接比绝对值。</li>
      </ol>
    </section>

    <div class="toolbar">
      <div class="toolbar-group">
        <span class="toolbar-label">数据库</span>
        <div class="tabs" role="tablist" aria-label="数据库" id="db-tabs">
          {"".join(db_buttons)}
        </div>
      </div>
      <div class="toolbar-group">
        <span class="toolbar-label">数据规模</span>
        <div class="tabs" role="tablist" aria-label="数据规模" id="n-tabs">
          {"".join(n_buttons)}
        </div>
      </div>
    </div>

    <div id="content"></div>

    <footer>
      <p>由 CI 从 Criterion 结果生成。需要 Slope / 置信区间等细节时，见
        <a href="report/index.html">详细统计（Criterion）</a>。</p>
    </footer>
  </div>

  <script>
    const OPS = {ops_json};
    const DATA = {data_json};
    const DEFAULT_DB = {json.dumps(default_db)};
    const DEFAULT_N = {json.dumps(default_n)};

    let currentDb = DEFAULT_DB;
    let currentN = DEFAULT_N;

    function fmtMs(v) {{
      return v.toFixed(1) + " ms";
    }}

    function fmtRel(v, best) {{
      if (v === best) return {{ text: "最快", best: true }};
      const pct = ((v / best) - 1) * 100;
      return {{ text: "+" + Math.round(pct) + "%", best: false }};
    }}

    function renderOp(op, rows) {{
      if (!rows || Object.keys(rows).length === 0) {{
        return "";
      }}
      const values = Object.values(rows);
      const best = Math.min(...values);
      const worst = Math.max(...values);
      const sorted = Object.entries(rows).sort((a, b) => a[1] - b[1]);

      const body = sorted.map(([orm, ms]) => {{
        const rel = fmtRel(ms, best);
        const width = worst > 0 ? (ms / worst) * 100 : 0;
        const fastClass = ms === best ? ' class="fastest"' : "";
        const relClass = rel.best ? "rel best" : "rel";
        return (
          "<tr" + fastClass + ">" +
            '<td class="orm">' + orm + "</td>" +
            '<td class="ms">' + fmtMs(ms) + "</td>" +
            '<td class="' + relClass + '">' + rel.text + "</td>" +
            '<td class="bar-cell"><div class="bar-track"><div class="bar-fill" style="width:' +
              width.toFixed(1) + '%"></div></div></td>' +
          "</tr>"
        );
      }}).join("");

      return (
        '<section class="op">' +
          '<div class="op-head">' +
            '<span class="op-label">' + op.title + "</span>" +
            '<span class="op-desc">' + op.desc + "</span>" +
          "</div>" +
          "<table>" +
            "<thead><tr>" +
              "<th>ORM</th><th>耗时</th><th>相对最快</th><th>示意</th>" +
            "</tr></thead>" +
            "<tbody>" + body + "</tbody>" +
          "</table>" +
        "</section>"
      );
    }}

    function render() {{
      const el = document.getElementById("content");
      const dbData = DATA[currentDb];
      if (!dbData || !dbData[currentN]) {{
        el.innerHTML = '<p class="empty">当前选择下暂无基准数据。</p>';
        return;
      }}
      const sizeData = dbData[currentN];
      const html = OPS.map((op) => renderOp(op, sizeData[op.id])).join("");
      el.innerHTML = html || '<p class="empty">当前选择下暂无基准数据。</p>';
    }}

    function selectDb(db) {{
      currentDb = db;
      document.querySelectorAll("#db-tabs button").forEach((btn) => {{
        btn.setAttribute("aria-selected", btn.dataset.db === db ? "true" : "false");
      }});
      render();
    }}

    function selectN(n) {{
      currentN = n;
      document.querySelectorAll("#n-tabs button").forEach((btn) => {{
        btn.setAttribute("aria-selected", btn.dataset.n === n ? "true" : "false");
      }});
      render();
    }}

    document.querySelectorAll("#db-tabs button:not([disabled])").forEach((btn) => {{
      btn.addEventListener("click", () => selectDb(btn.dataset.db));
    }});
    document.querySelectorAll("#n-tabs button").forEach((btn) => {{
      btn.addEventListener("click", () => selectN(btn.dataset.n));
    }});

    render();
  </script>
</body>
</html>
"""


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print(f"用法: {argv[0]} <criterion_root>", file=sys.stderr)
        return 2
    root = Path(argv[1])
    data = collect(root)
    if not data:
        print(f"警告: 在 {root} 未找到可用的 estimates.json", file=sys.stderr)

    out = root / "index.html"
    out.write_text(render_html(data), encoding="utf-8")
    n_pairs = sum(len(sizes) for sizes in data.values())
    print(f"wrote {out} ({n_pairs} db×n 组)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
