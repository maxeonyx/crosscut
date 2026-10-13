//! `crosscut/report.html`: the map with each concern's definition and what its reviews
//! found, as one page. It is generated from the files, so it can be rebuilt at any time.

use std::path::{Path, PathBuf};

use serde_json::json;

use crate::model;

pub fn write(root: &Path) -> Result<PathBuf, String> {
    let projects = model::projects(root)?;
    let concerns = model::concerns(root)?;
    let data = json!({
        "projects": projects.iter().map(|p| &p.name).collect::<Vec<_>>(),
        "concerns": concerns.iter().map(|c| {
            let observed = c.observations();
            let decided = c.decisions();
            json!({
                "slug": c.slug,
                "definition": c.definition,
                "cells": projects.iter().map(|p| {
                    let o = observed.iter().find(|o| o.project == p.name);
                    let d = decided.iter().find(|d| d.project == p.name);
                    let findings = std::fs::read_to_string(
                        c.dir.join("findings").join(format!("{}.md", p.name)),
                    ).ok();
                    json!({
                        "observed": o.map(|o| &o.status),
                        "since": o.map(|o| &o.since),
                        "evidence": o.map(|o| &o.evidence),
                        "decided": d.map(|d| &d.status),
                        "reason": d.map(|d| &d.reason),
                        "findings": findings,
                    })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
        "declined": std::fs::read_to_string(root.join("crosscut/declined.md")).ok(),
    });
    let data = data.to_string().replace("</", "<\\/");
    let path = root.join("crosscut/report.html");
    crate::discover::write(&path, &PAGE.replace("/*DATA*/null", &data))?;
    Ok(path)
}

const PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>CrossCut map</title>
<script src="https://cdnjs.cloudflare.com/ajax/libs/marked/12.0.2/marked.min.js"></script>
<style>
:root{--bg:#f6f5f1;--card:#fff;--ink:#1c1b19;--muted:#6b6862;--line:#dedad2;--yes:#e6eef2;--yes-ink:#2f5d73;--partly:#f6ecd6;--partly-ink:#8a5a12;--missing:#f3dcc4;--missing-ink:#8a3d0b;--na:#efede8;--na-ink:#8d8a83;--accent:#2f5d73}
@media (prefers-color-scheme:dark){:root{--bg:#141413;--card:#1d1c1a;--ink:#ecebe7;--muted:#a19e97;--line:#34322e;--yes:#1f2e36;--yes-ink:#9cc3d6;--partly:#3a2f1b;--partly-ink:#e7bf7a;--missing:#40291a;--missing-ink:#f0a46e;--na:#262522;--na-ink:#8d8a83;--accent:#9cc3d6}}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--ink);font:15px/1.55 ui-sans-serif,system-ui,-apple-system,"Segoe UI",sans-serif;padding:0 16px 80px}
main{max-width:1000px;margin:0 auto}
h1{font-size:28px;margin:36px 0 4px}
.lede{color:var(--muted);margin:0 0 24px}
.grid{overflow-x:auto;background:var(--card);border:1px solid var(--line);border-radius:8px;padding:12px}
table{border-collapse:separate;border-spacing:4px;width:100%}
th{font:600 12px/1.2 ui-monospace,monospace;color:var(--muted);text-align:left;padding:4px 6px;vertical-align:bottom}
td.rule{font-size:14px;padding:4px 6px;min-width:260px}
td.rule a{color:inherit;text-decoration:none}
td.rule a:hover{text-decoration:underline}
.s{display:inline-block;font:600 12px/1 ui-monospace,monospace;padding:6px 8px;border-radius:4px;white-space:nowrap}
.s-yes{background:var(--yes);color:var(--yes-ink)}.s-partly{background:var(--partly);color:var(--partly-ink)}.s-missing{background:var(--missing);color:var(--missing-ink)}
.s-na,.s-unknown,.s-deferred,.s-none{background:var(--na);color:var(--na-ink)}
section.concern{background:var(--card);border:1px solid var(--line);border-radius:8px;padding:20px 22px;margin-top:22px}
section.concern h2{font-size:20px;line-height:1.3;margin:0 0 8px}
.def{color:var(--muted);font-size:14px}
.def h1{display:none}.def h2{font-size:12px;letter-spacing:.08em;text-transform:uppercase;margin:16px 0 6px}
details summary{cursor:pointer;color:var(--accent);font-size:13px;margin:6px 0}
.cell{border-top:1px solid var(--line);padding-top:12px;margin-top:14px}
.cell h3{font-size:15px;margin:0 0 6px;display:flex;gap:10px;align-items:center;flex-wrap:wrap}
.evidence{color:var(--muted);margin:0 0 6px}
.findings code{font-size:13px;background:var(--na);padding:1px 4px;border-radius:3px}
.findings ul{padding-left:20px}
.declined{color:var(--muted);font-size:14px}
</style>
</head>
<body>
<main>
<h1>CrossCut map</h1>
<p class="lede">Each row is a concern: a rule worth keeping visible. Each cell is what the latest review saw in that project, or what a person decided. Nothing here is a gate or a score.</p>
<div class="grid"><table id="grid"></table></div>
<div id="concerns"></div>
<div id="declined"></div>
</main>
<script>
const D=/*DATA*/null;
const md=t=>window.marked?marked.parse(t||""):"<pre>"+(t||"").replace(/[&<]/g,c=>c=="&"?"&amp;":"&lt;")+"</pre>";
const esc=t=>(t||"").replace(/[&<>"]/g,c=>({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;"}[c]));
const title=c=>(c.definition.match(/^# (.*)$/m)||[,c.slug])[1];
const status=x=>x.decided||x.observed||"none";
const pill=x=>`<span class="s s-${status(x).replace("/","")}">${esc(x.decided?status(x)+" (decided)":status(x)=="none"?"not reviewed":status(x))}</span>`;
let g=`<tr><th></th>${D.projects.map(p=>`<th>${esc(p)}</th>`).join("")}</tr>`;
for(const c of D.concerns)g+=`<tr><td class="rule"><a href="#${c.slug}">${esc(title(c))}</a></td>${c.cells.map(x=>`<td>${pill(x)}</td>`).join("")}</tr>`;
document.getElementById("grid").innerHTML=g;
document.getElementById("concerns").innerHTML=D.concerns.map(c=>`<section class="concern" id="${c.slug}">
<h2>${esc(title(c))}</h2>
<details><summary>Why, and the stories</summary><div class="def">${md(c.definition)}</div></details>
${c.cells.map((x,i)=>`<div class="cell"><h3>${esc(D.projects[i])} ${pill(x)}</h3>
${x.reason?`<p class="evidence">Decided: ${esc(x.reason)}</p>`:""}
${x.evidence?`<p class="evidence">${esc(x.evidence)}${x.since?` <small>(since ${esc(x.since)})</small>`:""}</p>`:""}
${x.findings?`<div class="findings">${md(x.findings)}</div>`:""}</div>`).join("")}
</section>`).join("");
if(D.declined)document.getElementById("declined").innerHTML=`<section class="concern declined">${md(D.declined)}</section>`;
</script>
</body>
</html>
"##;
