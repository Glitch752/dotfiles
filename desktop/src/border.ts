import { formatHex, lerp, parseHex, converter, Oklch } from 'culori';
import { BorderState, Path, Rectangle } from './geom';
import { barThickness, nonBorderBarThickness } from './main';
import { debugLog } from './utils';

function drawPathWithRoundedCorners(ctx: CanvasRenderingContext2D, path: Path, r: number) {
    if (path.isEmpty() || path.length < 3) {
        return;
    }

    path.unclose();
    const count = path.length;

    for (let i = 0; i <= count; i++) {
        // Use modulo to wrap around the path points
        const prev = path.points[(i - 1 + count) % count];
        const cur = path.points[i % count];
        const next = path.points[(i + 1) % count];

        // The effective radius is the given radius or half the distance to the
        // next or previous point, whichever is smaller.
        const distPrev = prev.sub(cur).len();
        const distNext = next.sub(cur).len();
        const radius = Math.min(r, distPrev / 2.0, distNext / 2.0);

        // Find vectors pointing from the corner towards the adjacent points
        const prevNorm = prev.sub(cur).unit();
        const nextNorm = next.sub(cur).unit();

        if (i === 0) {
            // For the first point, move to the start of the first arc
            const start = cur.add(prevNorm.mul(radius));
            ctx.moveTo(start.x, start.y);
        }
        
        // After the last vertex, we just need to draw a line to the start of the first arc
        if (i === count) {
            const end = path.points[0].add(prevNorm.mul(radius));
            ctx.lineTo(end.x, end.y);
            continue;
        }

        const p1 = cur.add(prevNorm.mul(radius));
        const p2 = cur.add(nextNorm.mul(radius));
        ctx.lineTo(p1.x, p1.y);
        ctx.arcTo(cur.x, cur.y, p2.x, p2.y, radius);
    }
}


/**
 * Interpolates two angles in degrees, ensuring the shortest path is taken.
 */
function interpolateDegreesShorter(start: number, end: number, t: number): number {
    let delta = (end - start) % 360;
    if (delta > 180) {
        delta -= 360;
    } else if (delta < -180) {
        delta += 360;
    }
    return start + t * delta;
}

const toOklch = converter('oklch');

/**
 * Linearly interpolates two Oklch colors.
 */
function lerpOklch(startColor: Oklch, endColor: Oklch, t: number): Oklch {
    // Culori's `lerp` function doesn't handle hue shortest-path interpolation correctly by default.
    // We implement it manually.
    const l = lerp(startColor.l, endColor.l, t);
    const c = lerp(startColor.c, endColor.c, t);
    const h = interpolateDegreesShorter(startColor.h || 0, endColor.h || 0, t);
    return { mode: 'oklch', l, c, h };
}


export class BorderRenderer {
    private canvas: HTMLCanvasElement;
    private ctx: CanvasRenderingContext2D;

    private state: BorderState;

    // Drawing properties
    private backgroundColor: string;
    private cornerRadius: number;
    private borderThickness: number;
    private gradientColors: string[];
    private gradient: CanvasGradient;

    constructor(canvas: HTMLCanvasElement, ctx: CanvasRenderingContext2D) {
        this.canvas = canvas;
        this.ctx = ctx;

        this.state = new BorderState();

        // Get colors from :root in CSS
        const root = document.querySelector(":root") as HTMLElement;
        this.backgroundColor = getComputedStyle(root).getPropertyValue("--background");

        this.cornerRadius = 16;
        this.borderThickness = 2;
        this.gradientColors = [];

        this.gradient = null as unknown as CanvasGradient; // calculateGradient will set this; silences Typescript

        this.configureDrawingProperties();
        this.configureResizeHandler();
        this.calculateGradient();
    }

    private configureDrawingProperties() {
        // Get colors from :root in CSS
        const root = document.querySelector(":root") as HTMLElement;
        const gradientStart = getComputedStyle(root).getPropertyValue("--gradient-start");
        const gradientEnd = getComputedStyle(root).getPropertyValue("--gradient-end");

        // Since the canvas context doesn't support Oklch interpolation, we pre-calculate
        // a series of RGB colors and use them to create a linear gradient.
        const startColor = toOklch(parseHex(gradientStart))!;
        const endColor = toOklch(parseHex(gradientEnd))!;

        const stops = 20;
        for (let i = 0; i < stops; i++) {
            const t = i / (stops - 1); // Normalize to [0, 1]
            const interpolated = lerpOklch(startColor, endColor, t);
            this.gradientColors.push(formatHex(interpolated));
        }
    }

    private calculateGradient() {
        this.gradient = this.ctx.createLinearGradient(0, window.innerHeight, window.innerWidth, 0);
        this.gradientColors.forEach((color, i) => {
            const offset = i / (this.gradientColors.length - 1);
            this.gradient.addColorStop(offset, color);
        });
    }

    public draw() {
        const path = this.state.computeBorderPath();
        
        // Draw the borders
        // The stroke is centered on the path, so we use twice the thickness
        this.ctx.lineWidth = this.borderThickness * 2;
        this.ctx.strokeStyle = this.gradient;
        
        this.ctx.beginPath();
        drawPathWithRoundedCorners(this.ctx, path, this.cornerRadius);
        this.ctx.closePath();
        this.ctx.stroke();

        // We fill the entire canvas, then "punch out" the center shape using the
        // 'evenodd' fill rule. This leaves only the background area filled.
        this.ctx.fillStyle = this.backgroundColor;
        
        this.ctx.beginPath();
        this.ctx.rect(0, 0, window.innerWidth, window.innerHeight);
        drawPathWithRoundedCorners(this.ctx, path, this.cornerRadius);
        this.ctx.closePath();
        this.ctx.fill('evenodd');

        // this.state.debugDraw(this.ctx, false);
    }

    private configureResizeHandler() {
        const resizeObserver = new ResizeObserver(entries => {
            for (const entry of entries) {
                // Use device pixel ratio for sharper rendering
                const dpr = window.devicePixelRatio || 1;
                const { width, height } = entry.contentRect;

                this.canvas.width = Math.round(width * dpr);
                this.canvas.height = Math.round(height * dpr);

                this.ctx.scale(dpr, dpr);

                this.state.setBorderRect(Rectangle.filledOutward(
                    barThickness, barThickness,
                    window.innerWidth - nonBorderBarThickness,
                    window.innerHeight - nonBorderBarThickness
                ));

                this.calculateGradient();

                this.draw(); // Redraw after resizing
            }
        });

        resizeObserver.observe(this.canvas);
    }

    public setWidgetRectangles(rectangles: Rectangle[]) {
        this.state.setWidgetRectangles(rectangles);
    }
}