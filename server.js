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
  const headers = {
    "Content-Type": TYPES[ext] || "application/octet-stream",
    "Cache-Control": ext === ".html" ? "no-cache" : "public, max-age=3600",
    "Vary": "Accept-Encoding",
  };
  const stream = fs.createReadStream(file);
  if (req.method === "HEAD") return send(res, 200, headers, undefined);
  if (COMPRESSIBLE.has(ext) && /\bgzip\b/.test(req.headers["accept-encoding"] || "")) {
    res.writeHead(200, { ...BASE_HEADERS, ...headers, "Content-Encoding": "gzip" });
    stream.pipe(zlib.createGzip()).pipe(res);
  } else {
    res.writeHead(200, { ...BASE_HEADERS, ...headers });
    stream.pipe(res);
  }
}).listen(PORT, "0.0.0.0", () => console.log(`Outcrops Odyssey web on :${PORT}`));
