import http from 'http';
import fs from 'fs';
import path from 'path';

const PORT = 8080;
const PKG_DIR = path.join(process.cwd(), 'pkg');

const mimeTypes = {
    '.html': 'text/html',
    '.js': 'text/javascript',
    '.wasm': 'application/wasm',
    '.mjs': 'text/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpeg',
    '.svg': 'image/svg+xml',
};

const server = http.createServer((req, res) => {
    let filePath = '.' + req.url;
    if (filePath === './') {
        filePath = './index.html';
    }

    // If requesting from root, serve from pkg directory
    const fullPath = path.join(PKG_DIR, path.basename(filePath));

    const extname = String(path.extname(fullPath)).toLowerCase();
    const contentType = mimeTypes[extname] || 'application/octet-stream';

    fs.readFile(fullPath, (error, content) => {
        if (error) {
            if (error.code === 'ENOENT') {
                res.writeHead(404, { 'Content-Type': 'text/html' });
                res.end('<h1>404 Not Found</h1>', 'utf-8');
            } else {
                res.writeHead(500);
                res.end(`Server Error: ${error.code}`, 'utf-8');
            }
        } else {
            // Set proper MIME type for files
            res.writeHead(200, {
                'Content-Type': contentType,
            });
            res.end(content, 'utf-8');
        }
    });
});

server.listen(PORT, () => {
    console.log(`\n🎮 Bomberman 2D dev server running at:`);
    console.log(`   http://localhost:${PORT}\n`);
    console.log('Press Ctrl+C to stop\n');
});
