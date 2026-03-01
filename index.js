import init, { Renderer } from "./pkg/hello_wasm.js";

async function run() {
  const wasm = await init();

  const renderer = new Renderer();

  const canvas = document.getElementById("myCanvas");
  const ctx = canvas.getContext("2d");

  const width = 300;
  const height = 300;

  const ptr = renderer.buffer_ptr();
  const pixels = new Uint8ClampedArray(
    wasm.memory.buffer,
    ptr,
    width * height * 4,
  );

  const imageData = new ImageData(pixels, width, height);
  const fps = 60;
  const frameTime = 1000 / fps;
  let lastTime = 0;

  function render(currentTime) {
    if (currentTime - lastTime >= frameTime) {
    renderer.update(); // Rust mutates internal Vec
    ctx.putImageData(imageData, 0, 0);
    lastTime = currentTime;
    
    
    }
    requestAnimationFrame(render);
  }

  requestAnimationFrame(render);
}
run();
