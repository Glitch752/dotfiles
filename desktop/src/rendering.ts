import { BorderRenderer } from "./border";
import { Rectangle } from "./geom";
import { initializePopups } from "./popups/popups";

let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null;
let renderer: BorderRenderer | null = null;

export function init() {
    canvas = document.getElementById("canvas") as HTMLCanvasElement | null;
    ctx = canvas?.getContext("2d") ?? null;

    if(!canvas || !ctx) {
        console.error("Failed to get canvas context!");
        return;
    }

    renderer = new BorderRenderer(canvas, ctx);

    // This is a bit of a hack, but a reliable one.
    // ResizeObserver is always faster than 2 requestAnimationFrames per the spec,
    // so we just wait until then to render the first time. Hacky, but reliable.
    requestAnimationFrame(() => {
        requestAnimationFrame(() => {
            animate(0, []);

            initializePopups(animate);
        });
    });
}

function animate(_elapsed: number, rects: Rectangle[]) {
    if(!canvas || !ctx || !renderer) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    renderer.setWidgetRectangles(rects);
    renderer.draw();
}