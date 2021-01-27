declare module 'spritesmith' {
  export interface RunOptions {
    src: string[];
  }
  export interface RunResult {
    image: Buffer;
    coordinates: { [Key in string]: { x: number; y: number; width: number; height: number } };
    properties: { width: number; height: number };
  }
  export function run(options: RunOptions, cb: (err: Error | undefined, result: RunResult) => void);
}
