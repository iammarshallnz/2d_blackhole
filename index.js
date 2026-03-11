import init, { Renderer } from "./pkg/blackhole.js";

async function run() {
  const wasm = await init();
  const renderer = new Renderer();

  const canvas = document.getElementById("myCanvas");
  const ctx = canvas.getContext("2d");
  const width = 300;
  const height = 300;

  canvas.addEventListener("click", (event) => {
    const rect = canvas.getBoundingClientRect();

    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;

    const x = (event.clientX - rect.left) * scaleX;
    const y = (event.clientY - rect.top) * scaleY;
    try {
      renderer.add_ray_from_click(x, y);
    } catch (e) {
      console.error("Simulation error:", e.message);
    }
  });

  const slider = document.getElementById("massSlider");
  const massValue = document.getElementById("massValue");

  slider.addEventListener("input", () => {
    const mass36 = parseFloat(slider.value);
    massValue.textContent = mass36;

    // Convert 10^36 kg to real value
    const mass = mass36 * 1e36;

    try {
      renderer.set_blackhole_mass(mass);
    } catch (e) {
      console.error("Simulation error:", e.message);
    }
  });

  const fps = 60;
  const frameTime = 1000 / fps;
  let lastTime = 0;

  function render(currentTime) {
    if (currentTime - lastTime >= frameTime) {
      renderer.update(); // Rust mutates internal Vec

      const ptr = renderer.buffer_ptr();

      const pixels = new Uint8ClampedArray(
        wasm.memory.buffer,
        ptr,
        width * height * 4,
      );

      const imageData = new ImageData(pixels, width, height);
      ctx.putImageData(imageData, 0, 0);
      lastTime = currentTime;
    }
    requestAnimationFrame(render);
  }
  requestAnimationFrame(render);
}
run();
