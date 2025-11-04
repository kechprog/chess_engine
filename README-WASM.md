# Chess Engine - WebAssembly Deployment Guide

This guide explains how to build and deploy the Chess Engine as a WebAssembly application that runs in the browser.

## Prerequisites

Before building the WASM version, you need to install the following tools:

### 1. Rust Toolchain

Make sure you have Rust installed. If not, get it from [rustup.rs](https://rustup.rs/).

### 2. WASM Target

Add the WebAssembly compilation target to your Rust installation:

```bash
rustup target add wasm32-unknown-unknown
```

### 3. wasm-pack

Install wasm-pack, which handles the WASM build process:

```bash
cargo install wasm-pack
```

### 4. HTTP Server (for local testing)

You'll need a local HTTP server to test the application. Choose one of these options:

**Option A: miniserve (recommended)**
```bash
cargo install miniserve
```

**Option B: Python 3**
Python 3 comes with a built-in HTTP server. Most systems have Python pre-installed.

## Building for WASM

### Linux/Mac

```bash
# Make the build script executable
chmod +x build.sh

# Build the WASM module
./build.sh
```

### Windows

```batch
# Run the build script
build.bat
```

The build process will:
1. Check for required tools (wasm-pack, wasm32 target)
2. Compile the Rust code to WebAssembly
3. Generate JavaScript bindings
4. Output everything to the `./pkg` directory

## Running Locally

### Quick Start (Linux/Mac)

```bash
# Make the serve script executable
chmod +x serve.sh

# Build and start the development server
./serve.sh
```

### Quick Start (Windows)

```batch
# Build and start the development server
serve.bat
```

The serve script will:
1. Build the WASM module if it doesn't exist
2. Start a local HTTP server on port 8080
3. Provide the URL to open in your browser

Then open your browser and navigate to:
```
http://localhost:8080
```

### Manual Server Setup

If you prefer to use a different server or port:

**Using Python:**
```bash
python3 -m http.server 8080
```

**Using miniserve:**
```bash
miniserve . -p 8080 --index index.html
```

**Using Node.js http-server:**
```bash
npx http-server -p 8080
```

## Project Structure

After building, your project will have these key files:

```
chess_engine-wasm/
├── index.html              # Main HTML page
├── pkg/                    # Generated WASM files (not in git)
│   ├── chess_engine.js     # JavaScript bindings
│   ├── chess_engine_bg.wasm # Compiled WebAssembly
│   └── ...
├── build.sh / build.bat    # Build scripts
├── serve.sh / serve.bat    # Development server scripts
└── src/                    # Rust source code
```

## Deployment

### Building for Production

For production deployment, the WASM files are already optimized with:
- Size optimization (`opt-level = "z"`)
- Link-time optimization (LTO)
- Single codegen unit
- Panic abort mode

These settings are configured in `Cargo.toml` under `[profile.release]`.

### Hosting Options

#### Static Site Hosting

The application is a static site and can be deployed to any static hosting service:

1. **GitHub Pages**
   - Push your code to GitHub
   - Build locally: `./build.sh` or `build.bat`
   - Commit the `pkg/` directory (remove it from .gitignore first)
   - Enable GitHub Pages in repository settings
   - Set source to the main branch

2. **Netlify**
   - Connect your repository
   - Build command: `wasm-pack build --target web --release`
   - Publish directory: `.` (root)

3. **Vercel**
   - Import your repository
   - No build configuration needed (deploy as static)
   - Vercel will serve the files directly

4. **Cloudflare Pages**
   - Connect repository
   - Build command: `wasm-pack build --target web --release`
   - Build output directory: `/`

#### Manual Deployment

For manual deployment to any web server:

1. Build the project: `./build.sh` or `build.bat`
2. Upload these files to your server:
   - `index.html`
   - `pkg/` directory (entire folder)
   - Any other assets

3. Ensure your server:
   - Serves files with correct MIME types
   - Serves `.wasm` files as `application/wasm`
   - Has CORS enabled if loading from different origins

### MIME Types

Most servers handle these automatically, but if needed, ensure:
- `.wasm` files: `application/wasm`
- `.js` files: `application/javascript` or `text/javascript`
- `.html` files: `text/html`

## Development Tips

### Rebuilding

After making changes to the Rust code:

```bash
# Linux/Mac
./build.sh

# Windows
build.bat
```

Then refresh your browser (hard refresh with Ctrl+Shift+R or Cmd+Shift+R).

### Browser Console

Open your browser's developer console (F12) to see:
- Loading status
- Error messages
- Debug output from the application

### WASM File Size

The compiled WASM file is optimized for size but may still be several hundred KB. Consider:
- Using gzip/brotli compression on your server
- Implementing service workers for caching
- Loading the WASM module with streaming compilation

### Debugging

For debugging WASM applications:
1. Use `console_log` and `console_error_panic_hook` (already in dependencies)
2. Check browser console for Rust panic messages
3. Use browser developer tools to inspect network requests
4. Enable source maps in wasm-pack for better debugging

## Troubleshooting

### Build Errors

**Error: "wasm-pack not found"**
```bash
cargo install wasm-pack
```

**Error: "wasm32-unknown-unknown target not found"**
```bash
rustup target add wasm32-unknown-unknown
```

### Runtime Errors

**"Failed to load WASM module"**
- Make sure you built the project first
- Check that `pkg/` directory exists and contains `chess_engine.js` and `chess_engine_bg.wasm`
- Verify you're accessing via HTTP server (not file://)

**Blank screen**
- Open browser console (F12) to see errors
- Check network tab to verify all files are loading
- Ensure WASM file is being served with correct MIME type

**CORS errors**
- Make sure you're using an HTTP server, not opening index.html directly
- If loading from a different domain, ensure CORS headers are set

## Browser Compatibility

The Chess Engine WASM version requires:
- WebAssembly support (available in all modern browsers)
- ES6 modules support
- WebGPU support (for rendering)

Supported browsers:
- Chrome/Edge 113+
- Firefox 115+
- Safari 16.4+

## Performance Notes

- WASM compilation happens once on first load
- Subsequent loads use browser cache
- Performance is near-native speed
- Canvas rendering is hardware-accelerated via WebGPU

## Additional Resources

- [wasm-pack documentation](https://rustwasm.github.io/wasm-pack/)
- [Rust and WebAssembly book](https://rustwasm.github.io/book/)
- [WebGPU specification](https://www.w3.org/TR/webgpu/)
- [wgpu examples](https://github.com/gfx-rs/wgpu/tree/master/examples)
