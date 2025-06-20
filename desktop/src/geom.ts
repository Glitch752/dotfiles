// A small tolerance for floating-point comparisons.
const EPSILON: number = 1e-5;

/**
 * Represents a 2D point with floating-point coordinates.
 */
export class Point {
    public x: number;
    public y: number;

    constructor(x: number = 0, y: number = 0) {
        this.x = x;
        this.y = y;
    }

    /**
     * Calculates the magnitude (length) of the vector from the origin to this point.
     */
    len(): number {
        return Math.sqrt(this.x * this.x + this.y * this.y);
    }

    /**
     * Returns a new Point representing the unit vector of this point.
     * Returns a zero vector if the length is close to zero.
     */
    unit(): Point {
        const len = this.len();
        if (len < EPSILON) {
            return new Point(0, 0); // Avoid division by zero
        }
        return new Point(this.x / len, this.y / len);
    }

    /**
     * Calculates the 2D cross product (as a scalar) with another point.
     */
    cross(other: Point): number {
        return this.x * other.y - this.y * other.x;
    }

    /**
     * Calculates the angle of the point in radians from the positive X axis.
     */
    angle(): number {
        return Math.atan2(this.y, this.x);
    }

    /**
     * Subtracts another point from this point and returns a new Point.
     */
    sub(other: Point): Point {
        return new Point(this.x - other.x, this.y - other.y);
    }

    /**
     * Adds another point to this point and returns a new Point.
     */
    add(other: Point): Point {
        return new Point(this.x + other.x, this.y + other.y);
    }

    /**
     * Multiplies this point by a scalar and returns a new Point.
     */
    mul(scalar: number): Point {
        return new Point(this.x * scalar, this.y * scalar);
    }
    
    /**
     * Checks for equality with another point within a tolerance (EPSILON).
     */
    equals(other: Point): boolean {
        return Math.abs(this.x - other.x) < EPSILON && Math.abs(this.y - other.y) < EPSILON;
    }

    /**
     * Creates a string hash for use as a key in Maps or Sets.
     * It rounds coordinates to the nearest EPSILON to handle floating-point inaccuracies.
     */
    hash(): string {
        const x_rounded = Math.round(this.x / EPSILON);
        const y_rounded = Math.round(this.y / EPSILON);
        return `${x_rounded},${y_rounded}`;
    }
}

/**
 * Represents a sequence of connected points.
 */
export class Path {
    private points: Point[];

    constructor(points: Point[] = []) {
        this.points = points;
    }

    push(point: Point) {
        this.points.push(point);
    }

    get length(): number {
        return this.points.length;
    }

    isEmpty(): boolean {
        return this.points.length === 0;
    }

    last(): Point | undefined {
        return this.points[this.points.length - 1];
    }
    
    first(): Point | undefined {
        return this.points[0];
    }
    
    get(index: number): Point {
        return this.points[index];
    }

    /**
     * Closes the path by adding the first point to the end if it's not already closed.
     */
    close() {
        if (this.length > 1) {
            const first = this.first()!;
            const last = this.last()!;
            if (!first.equals(last)) {
                this.points.push(new Point(first.x, first.y));
            }
        }
    }

    /**
     * Uncloses the path by removing the last point if it's the same as the first.
     */
    unclose() {
        if (this.length > 1) {
            const first = this.first()!;
            const last = this.last()!;
            if (first.equals(last)) {
                this.points.pop();
            }
        }
    }
}

/**
 * Defines whether a rectangle's area is considered "filled" on its inside or outside.
 * - Inward: The inside of the rectangle is solid. Lines inside it should be pruned.
 * - Outward: The outside is solid (e.g., a hole). Lines outside it should be pruned.
 */
export enum RectanglePolarity {
    FilledInward,
    FilledOutward
}

/**
 * Represents an axis-aligned rectangle.
 */
export class Rectangle {
    center: Point;
    width: number;
    height: number;
    polarity: RectanglePolarity;

    constructor(center: Point, width: number, height: number, polarity: RectanglePolarity) {
        this.center = center;
        this.width = width;
        this.height = height;
        this.polarity = polarity;
    }
    
    static filledOutward(x0: number, y0: number, x1: number, y1: number): Rectangle {
        return new Rectangle(
            new Point((x0 + x1) / 2, (y0 + y1) / 2),
            x1 - x0,
            y1 - y0,
            RectanglePolarity.FilledOutward
        );
    }
    
    static filledInward(x0: number, y0: number, x1: number, y1: number): Rectangle {
        return new Rectangle(
            new Point((x0 + x1) / 2, (y0 + y1) / 2),
            x1 - x0,
            y1 - y0,
            RectanglePolarity.FilledInward
        );
    }

    static filledInwardCenter(center_x: number, center_y: number, width: number, height: number): Rectangle {
        return new Rectangle(
            new Point(center_x, center_y),
            width,
            height,
            RectanglePolarity.FilledInward
        );
    }

    static filledOutwardCenter(center_x: number, center_y: number, width: number, height: number): Rectangle {
        return new Rectangle(
            new Point(center_x, center_y),
            width,
            height,
            RectanglePolarity.FilledOutward
        );
    }

    /**
     * Returns the four corners of the rectangle.
     * [Top-Left, Top-Right, Bottom-Right, Bottom-Left]
     */
    getCorners(): [Point, Point, Point, Point] {
        const halfW = this.width / 2;
        const halfH = this.height / 2;
        return [
            new Point(this.center.x - halfW, this.center.y + halfH),
            new Point(this.center.x + halfW, this.center.y + halfH),
            new Point(this.center.x + halfW, this.center.y - halfH),
            new Point(this.center.x - halfW, this.center.y - halfH)
        ];
    }
    
    rectangleInt(): { x: number; y: number; width: number; height: number; } {
        const x = this.center.x - this.width / 2;
        const y = this.center.y - this.height / 2;
        return {
            x: Math.round(x),
            y: Math.round(y),
            width: Math.round(this.width),
            height: Math.round(this.height)
        };
    }
}

type LineSegment = [Point, Point];

class LineSegments {
    private segments: LineSegment[] = [];

    addRectangle(rect: Rectangle) {
        const corners = rect.getCorners();
        this.segments.push([corners[0], corners[1]]);
        this.segments.push([corners[1], corners[2]]);
        this.segments.push([corners[2], corners[3]]);
        this.segments.push([corners[3], corners[0]]);
    }

    /**
     * Prune line segments that are completely inside a "FilledInward" rectangle,
     * or completely outside a "FilledOutward" rectangle. Also cleans up
     * internal edges created at T-junctions.
     */
    pruneSegments(rect: Rectangle) {
        const x0 = rect.center.x - rect.width / 2;
        const y0 = rect.center.y - rect.height / 2;
        const x1 = rect.center.x + rect.width / 2;
        const y1 = rect.center.y + rect.height / 2;

        // Step 1: Prune segments fully inside/outside the rect.
        this.segments = this.segments.filter(([p1, p2]) => {
            if (rect.polarity === RectanglePolarity.FilledInward) {
                const isInside = (p: Point) =>
                    p.x > x0 + EPSILON && p.x < x1 - EPSILON &&
                    p.y > y0 + EPSILON && p.y < y1 - EPSILON;
                return !(isInside(p1) && isInside(p2));
            } else { // FilledOutward
                const isOutside = (p: Point) =>
                    p.x < x0 - EPSILON || p.x > x1 + EPSILON ||
                    p.y < y0 - EPSILON || p.y > y1 + EPSILON;
                return !(isOutside(p1) && isOutside(p2));
            }
        });

        // Step 2: For inward-filled rectangles, remove internal edges at T-junctions.
        if (rect.polarity !== RectanglePolarity.FilledInward) {
            return;
        }

        const endpointMap = new Map<string, LineSegment[]>();
        for (const seg of this.segments) {
            const [p1, p2] = seg;
            const hash1 = p1.hash();
            const hash2 = p2.hash();
            if (!endpointMap.has(hash1)) endpointMap.set(hash1, []);
            if (!endpointMap.has(hash2)) endpointMap.set(hash2, []);
            endpointMap.get(hash1)!.push(seg);
            endpointMap.get(hash2)!.push(seg);
        }

        const segmentsToRemove = new Set<LineSegment>();
        for (const segmentsAtPoint of endpointMap.values()) {
            if (segmentsAtPoint.length >= 3) {
                const junctionPoint = segmentsAtPoint[0][0].hash() === endpointMap.keys().next().value ? segmentsAtPoint[0][0] : segmentsAtPoint[0][1];
                
                // For a T-junction, find the two collinear segments and the one perpendicular "stem".
                // This is simpler for axis-aligned rectangles.
                const horizontal: LineSegment[] = [];
                const vertical: LineSegment[] = [];
                for(const seg of segmentsAtPoint) {
                    const other = seg[0].equals(junctionPoint) ? seg[1] : seg[0];
                    if (Math.abs(other.y - junctionPoint.y) < EPSILON) {
                        horizontal.push(seg);
                    } else {
                        vertical.push(seg);
                    }
                }

                let stem: LineSegment | null = null;
                if (horizontal.length === 1 && vertical.length >= 2) {
                    stem = horizontal[0];
                } else if (vertical.length === 1 && horizontal.length >= 2) {
                    stem = vertical[0];
                }

                if (stem) {
                    // Check if the stem's midpoint is inside the rectangle. If so, prune it.
                    const midPoint = stem[0].add(stem[1]).mul(0.5);
                    const isInside = midPoint.x > x0 + EPSILON && midPoint.x < x1 - EPSILON &&
                                     midPoint.y > y0 + EPSILON && midPoint.y < y1 - EPSILON;
                    if (isInside) {
                       segmentsToRemove.add(stem);
                    }
                }
            }
        }
        
        if (segmentsToRemove.size > 0) {
            this.segments = this.segments.filter(seg => !segmentsToRemove.has(seg));
        }
    }

    reset() {
        this.segments = [];
    }

    splitAtIntersections() {
        const segmentsIntersect = (segA: LineSegment, segB: LineSegment): Point | null => {
            const [p, p2] = segA;
            const r = p2.sub(p);
            const [q, q2] = segB;
            const s = q2.sub(q);

            const rxs = r.cross(s);
            if (Math.abs(rxs) < EPSILON) { // Parallel or collinear
                return null;
            }

            const qp = q.sub(p);
            const t = qp.cross(s) / rxs;
            const u = qp.cross(r) / rxs;

            // Intersects if t and u are between 0 and 1 (inclusive, with tolerance).
            const t_in = t >= -EPSILON && t <= 1.0 + EPSILON;
            const u_in = u >= -EPSILON && u <= 1.0 + EPSILON;
            
            if (t_in && u_in) {
                // Ignore intersections that occur at the endpoints of both segments,
                // as they are already connected.
                const t_is_endpoint = Math.abs(t) < EPSILON || Math.abs(t - 1.0) < EPSILON;
                const u_is_endpoint = Math.abs(u) < EPSILON || Math.abs(u - 1.0) < EPSILON;
                if (t_is_endpoint && u_is_endpoint) {
                    return null;
                }
                return p.add(r.mul(t));
            }
            return null;
        };

        const n = this.segments.length;
        if (n === 0) return;

        // Map from original segment index to a list of points on it (including new intersections).
        const splitPoints = new Map<number, Point[]>();
        for (let i = 0; i < n; i++) {
            splitPoints.set(i, [...this.segments[i]]);
        }
        
        // Find all intersection points.
        for (let i = 0; i < n; i++) {
            for (let j = i + 1; j < n; j++) {
                const intersection = segmentsIntersect(this.segments[i], this.segments[j]);
                if (intersection) {
                    splitPoints.get(i)!.push(intersection);
                    splitPoints.get(j)!.push(intersection);
                }
            }
        }
        
        // Rebuild segments list from the split points.
        const newSegments: LineSegment[] = [];
        for (let i = 0; i < n; i++) {
            const points = splitPoints.get(i)!;
            const p0 = this.segments[i][0];

            // Sort points along the segment's axis.
            points.sort((a, b) => {
                const da = a.sub(p0).len();
                const db = b.sub(p0).len();
                return da - db;
            });
            
            // Deduplicate points and create new segments.
            const uniquePoints: Point[] = [];
            if (points.length > 0) {
                uniquePoints.push(points[0]);
                for(let k = 1; k < points.length; k++) {
                    if (!points[k].equals(uniquePoints[uniquePoints.length - 1])) {
                        uniquePoints.push(points[k]);
                    }
                }
            }
            
            for (let k = 0; k < uniquePoints.length - 1; k++) {
                newSegments.push([uniquePoints[k], uniquePoints[k+1]]);
            }
        }
        
        this.segments = newSegments;
    }

    generatePath(): Path {
        if (this.segments.length === 0) {
            return new Path();
        }

        // Use a Set of the segments themselves for efficient removal.
        const unusedSegments = new Set<LineSegment>(this.segments);
        const startSegment = this.segments[0];

        const path = new Path([startSegment[0], startSegment[1]]);
        unusedSegments.delete(startSegment);

        while (unusedSegments.size > 0) {
            const lastPoint = path.last()!;
            let foundNext = false;
            
            for (const seg of unusedSegments) {
                const [p1, p2] = seg;
                if (p1.equals(lastPoint)) {
                    path.push(p2);
                    unusedSegments.delete(seg);
                    foundNext = true;
                    break;
                } else if (p2.equals(lastPoint)) {
                    path.push(p1);
                    unusedSegments.delete(seg);
                    foundNext = true;
                    break;
                }
            }

            if (!foundNext) break;
        }

        return path;
    }

    debugDraw(ctx: CanvasRenderingContext2D, usePath: boolean) {
        ctx.lineWidth = 2.0;

        if (usePath) {
            const path = this.generatePath();
            if (path.isEmpty()) return;

            ctx.strokeStyle = "black";
            ctx.beginPath();
            ctx.moveTo(path.get(0).x, path.get(0).y);
            for (let i = 1; i < path.length; i++) {
                ctx.lineTo(path.get(i).x, path.get(i).y);
            }
            ctx.stroke();
            return;
        }

        // Draw individual segments with independent colors
        for (let i = 0; i < this.segments.length; i++) {
            const [p1, p2] = this.segments[i];
            const hue = (i * 360 / this.segments.length) % 360;
            ctx.strokeStyle = `hsl(${hue}, 100%, 50%)`;
            ctx.beginPath();
            ctx.moveTo(p1.x, p1.y);
            ctx.lineTo(p2.x, p2.y);
            ctx.stroke();
        }
    }
}

/**
 * Manages the state and computation of a border from a set of rectangles.
 */
export class BorderState {
    private borderRect: Rectangle;
    private rectangles: Rectangle[];
    private lineSegments: LineSegments;

    constructor() {
        // Initialize with a default empty border.
        this.borderRect = Rectangle.filledOutward(0, 0, 0, 0);
        this.rectangles = [];
        this.lineSegments = new LineSegments();
    }

    setBorderRect(border: Rectangle) {
        this.borderRect = border;
        // The border rect is always the first in the list.
        if (this.rectangles.length === 0) {
            this.rectangles.push(this.borderRect);
        } else {
            this.rectangles[0] = this.borderRect;
        }
    }

    setWidgetRectangles(rectangles: Rectangle[]) {
        this.rectangles = [this.borderRect, ...rectangles];
    }
    
    computeBorderPath(): Path {
        const startTime = performance.now();
        this.lineSegments.reset();

        for (const rect of this.rectangles) {
            this.lineSegments.addRectangle(rect);
        }
        
        this.lineSegments.splitAtIntersections();

        // Prune segments against every rectangle.
        for (const rect of this.rectangles) {
            this.lineSegments.pruneSegments(rect);
        }

        const path = this.lineSegments.generatePath();
        const endTime = performance.now();
        console.log(`Computed paths in ${(endTime - startTime).toFixed(2)} ms`);
        return path;
    }

    debugDraw(ctx: CanvasRenderingContext2D, usePath: boolean) {
        this.lineSegments.debugDraw(ctx, usePath);
    }
}