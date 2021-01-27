export interface Point {
  x: number;
  y: number;
}

export interface Dimensions {
  width: number;
  height: number;
}

export interface Rectangle extends Point, Dimensions {
  p1: Point;
}

export interface Colour {
  r: number;
  g: number;
  b: number;
}

export function colourToHex({ r, g, b }: Colour): string {
  function hex(x: number) {
    return x.toString(16).padStart(2, '0');
  }
  return `#${hex(r)}${hex(g)}${hex(b)}`;
}

export function randomColour(): Colour {
  function rand() {
    return Math.round(Math.random() * 255);
  }
  return { r: rand(), g: rand(), b: rand() };
}

export enum SpriteType {
  Character,
  Image,
}

export interface Sprite {
  id: string;
  type: SpriteType;
  name: string;
  svg: string;
  colour: Colour;
  polygons: number[][];
  size: Dimensions;
}

export interface SpriteSheetEntry {
  polygons: number[][];
  position: Point;
  size: Dimensions;
  colour: Colour;
}
