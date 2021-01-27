declare module 'potrace' {
  export interface LoadImageOptions {
    background?: string;
    color?: string;
    threshold?: number;
  }
  export class Potrace {
    loadImage(path: string, cb: (err: Error) => void): void;
    getSVG(): string;
  }

  export function trace(path: string, options: LoadImageOptions, cb: (err: Error, svg: string) => void): void;
}
