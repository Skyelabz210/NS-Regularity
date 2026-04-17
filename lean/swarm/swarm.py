#!/usr/bin/env python3
"""NS Regularity Formalization Swarm — dispatcher.

Reads blueprint.json, picks pending leaves, dispatches each to one or more
model providers, patches Lean files, runs `lake build`, accepts or re-queues.

The `sorry` is a status marker, not a failure. Leaves that agents cannot close
stay `sorry` and get flagged for further decomposition.

Usage:
  swarm.py status                       # print DAG state
  swarm.py dispatch --leaf L1.1         # one leaf, default providers
  swarm.py dispatch --all-pending       # every pending leaf in parallel
  swarm.py dispatch --leaf L1.1 \\
           --provider kimi --provider qwen --provider deepseek
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

LEAN_ROOT = Path(__file__).resolve().parent.parent
BLUEPRINT = LEAN_ROOT / "blueprint.json"
CANDIDATE_DIR = LEAN_ROOT / "swarm" / "candidates"
LOG_DIR = LEAN_ROOT / "swarm" / "logs"

NVIDIA_URL = "https://integrate.api.nvidia.com/v1/chat/completions"
NVIDIA_MODELS = {
    "kimi":     "moonshotai/kimi-k2.5",
    "kimi-think": "moonshotai/kimi-k2-thinking",
    "qwen":     "qwen/qwen3.5-397b-a17b",
    "qwen-think": "qwen/qwen3-next-80b-a3b-thinking",
    "qwen-coder": "qwen/qwen3-coder-480b-a35b-instruct",
    "deepseek": "deepseek-ai/deepseek-v3.2",
}

SYSTEM_PROMPT = """You are a Lean 4 (v4.29.1, no mathlib) proof assistant.
The user will send you a Lean 4 file containing theorems whose bodies are `sorry`.
Your job: replace each `sorry` with a valid Lean 4 proof.

Hard rules:
- Return ONLY the complete updated Lean 4 file, nothing else.
- No markdown fences, no commentary, no explanation.
- Do not modify imports, definitions, or any other theorem's statement.
- If you cannot prove a goal, LEAVE ITS `sorry` IN PLACE. Do not introduce new `sorry`s.
- Keep each proof short (≤ 30 lines).

Tactics available without mathlib: rfl, decide, simp, unfold, omega, exact, apply,
intro, cases, induction, rw, show, change, have, use, constructor, refine, by_contra,
contradiction, Nat.rec, Fin.rec, List.rec.

Useful patterns (try in this order):
1. `by decide` — WORKS for any `∀ i : Fin 8, P i` when P is decidable.
   This is the FIRST thing to try on finite-type universal statements.
2. `by rfl` — if the statement is definitionally true.
3. `by unfold <def>; decide` — if a definition needs to unfold first.
4. `by intro i; fin_cases i <;> rfl` — enumerate Fin 8 cases.
5. `simp [sp, sn, Fin.add_def, Fin.ext_iff]` then `omega` — for arithmetic.

For sums over `List.finRange 8`, `decide` computes the value directly.
For List.foldl invariance lemmas, use a helper like
  `have h : List.finRange 8 = [0,1,2,3,4,5,6,7] := by decide`
then `simp [h]` followed by computation."""


def load_blueprint():
    return json.loads(BLUEPRINT.read_text())


def save_blueprint(bp):
    BLUEPRINT.write_text(json.dumps(bp, indent=2))


def find_leaf(bp, leaf_id):
    for leaf in bp["leaves"]:
        if leaf["id"] == leaf_id:
            return leaf
    raise KeyError(f"no leaf {leaf_id}")


def call_nvidia(model_key, file_content, timeout=240):
    import urllib.request
    key = os.environ.get("NVIDIA_API_KEY")
    if not key:
        raise RuntimeError("NVIDIA_API_KEY unset")
    body = {
        "model": NVIDIA_MODELS[model_key],
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": file_content},
        ],
        "temperature": 0.2,
        "top_p": 0.9,
        "max_tokens": 4096,
        "stream": False,
    }
    req = urllib.request.Request(
        NVIDIA_URL,
        data=json.dumps(body).encode(),
        headers={
            "Authorization": f"Bearer {key}",
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(req, timeout=timeout) as resp:
        data = json.loads(resp.read())
    return data["choices"][0]["message"]["content"]


def call_codex(file_content, timeout=180):
    """Shell out to `codex exec`. Requires `codex login` to be done."""
    full_prompt = SYSTEM_PROMPT + "\n\n---\n\n" + file_content
    result = subprocess.run(
        ["codex", "exec", "--skip-git-repo-check", full_prompt],
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    if result.returncode != 0:
        raise RuntimeError(f"codex exit {result.returncode}: {result.stderr[:500]}")
    return result.stdout


PROVIDERS = {
    "kimi":     lambda fc: call_nvidia("kimi", fc),
    "qwen":     lambda fc: call_nvidia("qwen", fc),
    "deepseek": lambda fc: call_nvidia("deepseek", fc),
    "codex":    call_codex,
}


def strip_fences(text):
    """Some models wrap output in ```lean ... ``` despite instructions."""
    text = text.strip()
    m = re.match(r"^```(?:lean4?)?\n(.*?)\n```\s*$", text, re.DOTALL)
    return m.group(1) if m else text


def lake_build(target=None):
    cmd = ["lake", "build"] + ([target] if target else [])
    env = os.environ.copy()
    env["PATH"] = f"{os.path.expanduser('~/.elan/bin')}:{env.get('PATH','')}"
    return subprocess.run(cmd, capture_output=True, text=True, cwd=LEAN_ROOT, env=env)


def count_sorries(text):
    return len(re.findall(r"\bsorry\b", text))


def dispatch_leaf(leaf, providers):
    leaf_id = leaf["id"]
    file_rel = leaf["file"]
    file_abs = LEAN_ROOT / file_rel
    if not file_abs.exists():
        return {"leaf": leaf_id, "status": "missing_file", "file": file_rel}

    original = file_abs.read_text()
    orig_sorries = count_sorries(original)
    if orig_sorries == 0:
        return {"leaf": leaf_id, "status": "already_closed"}

    ts = time.strftime("%Y%m%d-%H%M%S")
    logf = LOG_DIR / f"{leaf_id}.{ts}.log"
    logf.parent.mkdir(parents=True, exist_ok=True)
    log = open(logf, "w")
    log.write(f"leaf={leaf_id} file={file_rel} orig_sorries={orig_sorries}\n")

    for prov in providers:
        log.write(f"\n=== provider: {prov} ===\n")
        try:
            raw = PROVIDERS[prov](original)
            candidate = strip_fences(raw)
            log.write(f"raw_len={len(raw)} candidate_len={len(candidate)}\n")
        except Exception as e:
            log.write(f"call_failed: {type(e).__name__}: {e}\n")
            continue

        cand_path = CANDIDATE_DIR / f"{leaf_id}.{prov}.{ts}.lean"
        cand_path.parent.mkdir(parents=True, exist_ok=True)
        cand_path.write_text(candidate)

        cand_sorries = count_sorries(candidate)
        log.write(f"candidate_sorries={cand_sorries}\n")
        if cand_sorries >= orig_sorries:
            log.write("rejected: agent did not reduce sorry count\n")
            continue

        # Write candidate, try to build, revert if fails.
        file_abs.write_text(candidate)
        build = lake_build()
        log.write(f"build rc={build.returncode}\n")
        log.write(build.stdout[-2000:] + "\n--STDERR--\n" + build.stderr[-2000:] + "\n")
        if build.returncode == 0:
            log.close()
            return {"leaf": leaf_id, "status": "closed", "provider": prov,
                    "candidate": str(cand_path), "log": str(logf)}
        # revert and try next provider
        file_abs.write_text(original)
        log.write("reverted to original; trying next provider\n")

    log.close()
    return {"leaf": leaf_id, "status": "no_provider_closed", "log": str(logf)}


def cmd_status(args):
    bp = load_blueprint()
    print(f"blueprint: {bp['meta']['root_theorem']}")
    print(f"macros: {bp['stats']['macros_decomposed']}/{bp['stats']['macros_total']} decomposed")
    print(f"leaves: {bp['stats']['leaves_closed']} closed / {bp['stats']['leaves_pending']} pending / {bp['stats']['leaves_total']} total")
    print()
    for leaf in bp["leaves"]:
        f = LEAN_ROOT / leaf["file"]
        live = count_sorries(f.read_text()) if f.exists() else -1
        marker = "[CLOSED]" if live == 0 and leaf["status"] == "closed" else f"[{leaf['status']}]"
        print(f"  {leaf['id']:<8} {marker:<28} {leaf['file']}  live_sorries={live}")


def cmd_dispatch(args):
    bp = load_blueprint()
    if args.leaf:
        targets = [find_leaf(bp, args.leaf)]
    elif args.all_pending:
        targets = [lf for lf in bp["leaves"] if lf["status"] == "pending"]
    else:
        sys.exit("need --leaf ID or --all-pending")

    providers = args.provider or ["kimi", "deepseek", "qwen"]
    for p in providers:
        if p not in PROVIDERS:
            sys.exit(f"unknown provider: {p}")

    print(f"dispatching {len(targets)} leaf(ves) × providers={providers}")

    results = []
    with ThreadPoolExecutor(max_workers=args.concurrency) as ex:
        futs = {ex.submit(dispatch_leaf, lf, providers): lf["id"] for lf in targets}
        for fut in as_completed(futs):
            r = fut.result()
            print(f"  {r['leaf']}: {r['status']}" + (f" via {r.get('provider')}" if r.get("provider") else ""))
            results.append(r)

    # Update blueprint statuses
    closed_ids = {r["leaf"] for r in results if r["status"] == "closed"}
    for leaf in bp["leaves"]:
        if leaf["id"] in closed_ids:
            leaf["status"] = "closed"
    bp["stats"]["leaves_closed"] = sum(1 for lf in bp["leaves"] if lf["status"] == "closed")
    bp["stats"]["leaves_pending"] = sum(1 for lf in bp["leaves"] if lf["status"] == "pending")
    save_blueprint(bp)

    closed = sum(1 for r in results if r["status"] == "closed")
    print(f"\nsummary: {closed}/{len(results)} closed")


def main():
    p = argparse.ArgumentParser()
    sub = p.add_subparsers(dest="cmd")

    s = sub.add_parser("status")
    s.set_defaults(func=cmd_status)

    d = sub.add_parser("dispatch")
    d.add_argument("--leaf", help="single leaf id, e.g. L1.1")
    d.add_argument("--all-pending", action="store_true")
    d.add_argument("--provider", action="append", help="kimi|qwen|deepseek|codex (repeatable)")
    d.add_argument("--concurrency", type=int, default=4)
    d.set_defaults(func=cmd_dispatch)

    args = p.parse_args()
    if not getattr(args, "func", None):
        p.print_help(); sys.exit(1)
    args.func(args)


if __name__ == "__main__":
    main()
