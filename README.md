# Bomberman 2D

A classic Bomberman-style game built with Rust and the Bevy game engine.

## Features

- Tile-based map with walls and destructible boxes
- Player character with animated body, face, eyes, and hat
- Smooth player movement with collision detection
- Bomb placement based on facing direction
- Explosion system that destroys boxes
- Player death and respawn mechanic
- Strategic map layout with escape routes

## Controls

- **Arrow Keys / WASD**: Move player
- **Space / X**: Place bomb
- **Bombs explode**: After 2 seconds
- **Explosion range**: 2 tiles in each direction

## Running Locally

```bash
# Run native version
cargo run
```

## Web Deployment

This game can be deployed to the web using WebAssembly. To build for web:

1. Install wasm-pack:
```bash
cargo install wasm-pack
```

2. Build for web:
```bash
wasm-pack build --target web --release
```

3. Deploy the `pkg` or `dist` folder to Vercel, Netlify, or GitHub Pages.

## License

MIT
