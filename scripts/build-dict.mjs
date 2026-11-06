// Merges SIL khmerlbdict word-frequency files into a single sorted dictionary.
// Source: https://github.com/silnrsi/khmerlbdict (MIT) — files under src/*.txt
// Usage: node scripts/build-dict.mjs <sources-dir> [out-file]
import { readFileSync, writeFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const srcDir = process.argv[2] ?? "scripts/data-source";
const out = process.argv[3] ?? "data/khmer.lbdict.txt";

const freq = new Map();
for (const f of readdirSync(srcDir).filter((n) => n.endsWith(".txt"))) {
  let text = readFileSync(join(srcDir, f), "utf8").replace(/^\uFEFF/, "");
  for (const line of text.split("\n")) {
    const line0 = line.replace(/\r$/, "");
    if (!line0) continue;
    const [word, f] = line0.split("\t");
    if (!word) continue;
    const n = Number.parseInt(f, 10) || 1;
    freq.set(word, Math.max(freq.get(word) ?? 0, n));
  }
}

const kept = [];
for (const [word, n] of freq) {
  // Keep a word only if it is a single Khmer token: non-empty, no ZWSP, no
  // ASCII letters/digits, no spaces. Segmentation looks up single clusters
  // runs, so mixed-entry noise would never match and only bloats the package.
  if (!/[\u1780-\u17FF\u19E0-\u19FF\u200C]/u.test(word)) continue;
  if (/[\s\p{Zs}\u200B\x00-\x7F]/u.test(word)) continue;
  kept.push([word, n]);
}
kept.sort((a, b) => b[1] - a[1] || (a[0] < b[0] ? -1 : 1));

writeFileSync(out, kept.map(([w, n]) => `${w}\t${n}`).join("\n") + "\n", "utf8");
console.log(`${kept.length} entries -> ${out}`);
