declare module 'flatten-svg' {
  export interface Points {
    points: [x: number, y: number][];
  }
  export interface FlattenSVGOptions {
    maxError?: number;
  }
  export function flattenSVG(svgElement: any, options?: FlattenSVGOptions): Points[];
}
