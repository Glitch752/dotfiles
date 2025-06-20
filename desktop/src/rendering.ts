let canvas: HTMLCanvasElement | null = null;
let ctx: CanvasRenderingContext2D | null = null

export function init() {
    canvas = document.getElementById("canvas") as HTMLCanvasElement | null;
    ctx = canvas?.getContext("2d") ?? null;

    if(!canvas || !ctx) {
        console.error("Failed to get canvas context!");
        return;
    }

    resize();

    animate(0);
}

function animate(elapsed: number) {
    if(!canvas || !ctx) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    ctx.fillStyle = "red";
    ctx.fillRect(elapsed * 0.1, 0, 50, 50);

    requestAnimationFrame(animate);
}

export function resize() {
    if(!canvas) return;

    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
}