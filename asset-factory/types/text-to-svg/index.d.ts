declare module 'text-to-svg' {
  export interface TextToSvgOptions {
    x: number;
    y: number;
    fontSize: number;
    anchor: string;
    attributes: { fill: string; stroke: string };
  }
  export interface TextToSVG {
    getSVG(s: string, options?: TextToSvgOptions): string;
  }
  export function loadSync(path: string): TextToSVG;
}
