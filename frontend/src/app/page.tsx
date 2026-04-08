"use client";

import { useState, useEffect, useRef } from "react";

// ── Types ─────────────────────────────────────────────────────────────────────

interface CourseResult {
  title: string;
  description: string;
  url: string;
  credits: string;
  level: number;
  score: number;
  workload_score: number;
}

interface SearchResponse {
  results: CourseResult[];
  total: number;
  query_ms: number;
}

type Status = "idle" | "loading" | "success" | "error";

// ── Helpers ───────────────────────────────────────────────────────────────────

function workloadBadge(score: number): { label: string; cls: string } {
  if (score < 0.3) return { label: "Low Workload",    cls: "bg-green-800 text-green-200"  };
  if (score <= 0.6) return { label: "Medium Workload", cls: "bg-yellow-800 text-yellow-200" };
  return               { label: "Heavy Workload",    cls: "bg-red-800 text-red-200"      };
}

// ── Skeleton card ─────────────────────────────────────────────────────────────

function SkeletonCard() {
  return (
    <div className="rounded-xl border border-slate-700 bg-slate-800 p-5 animate-pulse">
      <div className="h-5 w-2/3 rounded bg-slate-600 mb-3" />
      <div className="h-3 w-full rounded bg-slate-700 mb-2" />
      <div className="h-3 w-4/5 rounded bg-slate-700 mb-4" />
      <div className="flex gap-2">
        <div className="h-5 w-16 rounded-full bg-slate-600" />
        <div className="h-5 w-20 rounded-full bg-slate-600" />
        <div className="h-5 w-24 rounded-full bg-slate-600" />
      </div>
    </div>
  );
}

// ── Result card ───────────────────────────────────────────────────────────────

function ResultCard({ result }: { result: CourseResult }) {
  const snippet =
    result.description.length > 200
      ? result.description.slice(0, 200) + "…"
      : result.description;

  const wl = workloadBadge(result.workload_score);
  const levelStr = result.level >= 400 ? "400-level" : "300-level";

  return (
    <a
      href={result.url}
      target="_blank"
      rel="noopener noreferrer"
      className="block rounded-xl border border-slate-700 bg-slate-800 p-5 transition hover:border-blue-500"
    >
      <h2 className="text-lg font-semibold text-white mb-1">
        EECS {result.title}
      </h2>
      <p className="text-sm text-slate-400 mb-3 leading-relaxed">{snippet}</p>

      <div className="flex flex-wrap items-center gap-2">
        <span className="rounded-full bg-blue-900 px-3 py-0.5 text-xs font-medium text-blue-200">
          {result.credits} Credits
        </span>
        <span className="rounded-full bg-slate-700 px-3 py-0.5 text-xs font-medium text-slate-300">
          {levelStr}
        </span>
        <span className={`rounded-full px-3 py-0.5 text-xs font-medium ${wl.cls}`}>
          {wl.label}
        </span>
        <span className="ml-auto text-xs text-slate-500">
          score: {result.score.toFixed(2)}
        </span>
      </div>
    </a>
  );
}

// ── Main page ─────────────────────────────────────────────────────────────────

export default function Home() {
  const [query, setQuery]       = useState("");
  const [submitted, setSubmitted] = useState("");
  const [status, setStatus]     = useState<Status>("idle");
  const [data, setData]         = useState<SearchResponse | null>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Debounced auto-search (300 ms)
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    if (!query.trim()) {
      setStatus("idle");
      setData(null);
      setSubmitted("");
      return;
    }
    debounceRef.current = setTimeout(() => runSearch(query.trim()), 300);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [query]);

  async function runSearch(q: string) {
    setSubmitted(q);
    setStatus("loading");
    setData(null);
    try {
      const res = await fetch(`/api/search?q=${encodeURIComponent(q)}`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const json: SearchResponse = await res.json();
      setData(json);
      setStatus("success");
    } catch {
      setStatus("error");
    }
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter" && query.trim()) {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      runSearch(query.trim());
    }
  }

  return (
    <main className="min-h-screen" style={{ background: "#0f172a" }}>
      <div className="mx-auto max-w-2xl px-4 py-16">

        {/* Header */}
        <div className="text-center mb-10">
          <h1 className="text-4xl font-bold text-white tracking-tight mb-2">
            UMich EECS Course Search
          </h1>
          <p className="text-slate-400 text-sm">
            Find upper-level CS courses · ranked by relevance + workload
          </p>
        </div>

        {/* Search bar */}
        <div className="flex gap-2 mb-8">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder='Try "easy ULCS low workload" or "machine learning"'
            className="flex-1 rounded-lg border border-slate-600 bg-slate-800 px-4 py-3 text-sm text-white placeholder-slate-500 outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
          />
          <button
            onClick={() => query.trim() && runSearch(query.trim())}
            className="rounded-lg bg-blue-600 px-5 py-3 text-sm font-medium text-white transition hover:bg-blue-500 active:bg-blue-700"
          >
            Search
          </button>
        </div>

        {/* Loading skeleton */}
        {status === "loading" && (
          <div className="space-y-4">
            <p className="text-xs text-slate-500 mb-2">Searching…</p>
            {[0, 1, 2].map((i) => <SkeletonCard key={i} />)}
          </div>
        )}

        {/* Error */}
        {status === "error" && (
          <div className="rounded-xl border border-red-800 bg-red-950 p-5 text-sm text-red-300">
            Could not reach the search API. Make sure{" "}
            <code className="rounded bg-red-900 px-1">cargo run -p api</code>{" "}
            is running on port&nbsp;3000.
          </div>
        )}

        {/* Empty results */}
        {status === "success" && data && data.results.length === 0 && (
          <p className="text-center text-slate-400 text-sm mt-8">
            No results for{" "}
            <span className="font-medium text-white">"{submitted}"</span>
          </p>
        )}

        {/* Results */}
        {status === "success" && data && data.results.length > 0 && (
          <>
            <p className="text-xs text-slate-500 mb-4">
              {data.total} result{data.total !== 1 ? "s" : ""} · {data.query_ms}ms
            </p>
            <div className="space-y-4">
              {data.results.map((r, i) => (
                <ResultCard key={i} result={r} />
              ))}
            </div>
          </>
        )}

      </div>
    </main>
  );
}
