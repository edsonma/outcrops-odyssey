// Minimal static server for the Rust/WebAssembly build. No dependencies.
// Serves ./public (override with STATIC_DIR) with correct MIME types
// (notably application/wasm) and gzip for text/wasm payloads.
const http = require("http");
const fs = require("fs");
const path = require("path");
const zlib = require("zlib");

const ROOT = path.resolve(__dirname, process.env.STATIC_DIR || "public");
const PORT = process.env.PORT || 3000;

const TYPES = {
  ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm", ".css": "text/css; charset=utf-8",
  ".json": "application/json", ".png": "image/png", ".jpg": "image/jpeg",
  ".ogg": "audio/ogg", ".wav": "audio/wav", ".mp3": "audio/mpeg",
  ".ttf": "font/ttf", ".txt": "text/plain; charset=utf-8", ".ico": "image/x-icon",
  ".data": "application/octet-stream", ".rb": "text/plain; charset=utf-8",
};
const COMPRESSIBLE = new Set([".html", ".js", ".wasm", ".css", ".json", ".txt", ".rb", ".data"]);

const BASE_HEADERS = {
  "X-Content-Type-Options": "nosniff",
};

function send(res, status, headers, body) {
  res.writeHead(status, { ...BASE_HEADERS, ...headers });
  res.end(body);
}

// Each compressible file is compressed once (brotli q11 + gzip 9) and kept in memory: the
// big .wasm costs a few seconds of CPU at the first request, then every visitor gets it for free.
const cache = new Map();
function load(file, ext) {
  const st = fs.statSync(file);
  const hit = cache.get(file);
  if (hit && hit.mtime === st.mtimeMs) return hit;
  const raw = fs.readFileSync(file);
  const entry = { mtime: st.mtimeMs, raw, etag: `"${st.size.toString(16)}-${Math.floor(st.mtimeMs).toString(16)}"` };
  if (COMPRESSIBLE.has(ext)) {
    entry.br = zlib.brotliCompressSync(raw, { params: { [zlib.constants.BROTLI_PARAM_QUALITY]: 11, [zlib.constants.BROTLI_PARAM_SIZE_HINT]: raw.length } });
    entry.gz = zlib.gzipSync(raw, { level: 9 });
  }
  cache.set(file, entry);
  return entry;
}

http.createServer((req, res) => {
  const url = new URL(req.url, "http://localhost");
  if (url.pathname === "/healthz") return send(res, 200, { "Content-Type": "text/plain" }, "ok");
  if (req.method !== "GET" && req.method !== "HEAD") return send(res, 405, {}, "Method Not Allowed");

  let rel;
  try { rel = decodeURIComponent(url.pathname); } catch { return send(res, 400, {}, "Bad Request"); }
  let file = path.normalize(path.join(ROOT, rel));
  if (file !== ROOT && !file.startsWith(ROOT + path.sep)) return send(res, 403, {}, "Forbidden");
  if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");

  if (!fs.existsSync(file)) {
    if (!fs.existsSync(path.join(ROOT, "index.html"))) {
      return send(res, 503, { "Content-Type": "text/html; charset=utf-8" },
        "<h1>Outcrops Odyssey</h1><p>No web build found. Build with the Dockerfile (it compiles the wasm).</p>");
    }
    return send(res, 404, { "Content-Type": "text/plain" }, "Not found");
  }

  const ext = path.extname(file).toLowerCase();
  const e = load(file, ext);
  const headers = {
    "Content-Type": TYPES[ext] || "application/octet-stream",
    // revalidate cheaply (ETag -> 304); html always, the rest may be reused for a day
    // The page, the JS glue and the .wasm are a matched set: a cached .wasm with a newer .js gives
    // "wasm.__wasm_bindgen_func_elem_N is not a function". So they are always revalidated
    // (ETag -> 304, costs nothing); only textures/models may be reused without asking.
    "Cache-Control": [".html", ".js", ".wasm"].includes(ext) ? "no-cache" : "public, max-age=86400, stale-while-revalidate=604800",
    "ETag": e.etag,
    "Vary": "Accept-Encoding",
    // lets the loading screen show a true percentage even though the body is compressed
    "X-Uncompressed-Length": String(e.raw.length),
    "Access-Control-Expose-Headers": "X-Uncompressed-Length",
  };
  if (req.headers["if-none-match"] === e.etag) return send(res, 304, headers);

  const accept = req.headers["accept-encoding"] || "";
  let body = e.raw;
  if (e.br && /\bbr\b/.test(accept)) { body = e.br; headers["Content-Encoding"] = "br"; }
  else if (e.gz && /\bgzip\b/.test(accept)) { body = e.gz; headers["Content-Encoding"] = "gzip"; }
  headers["Content-Length"] = String(body.length);
  return send(res, 200, headers, req.method === "HEAD" ? undefined : body);
}).listen(PORT, "0.0.0.0", () => {
  console.log(`Outcrops Odyssey web on :${PORT}`);
  // warm the cache so the first visitor does not pay for brotli q11
  for (const f of fs.readdirSync(ROOT)) {
    const ext = path.extname(f).toLowerCase();
    if (COMPRESSIBLE.has(ext)) try { load(path.join(ROOT, f), ext); } catch {}
  }
  console.log("compressed assets ready");
});
