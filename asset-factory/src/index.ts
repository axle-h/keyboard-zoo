import * as path from 'path';
import * as fs from 'fs';
import * as util from 'util';
import axios from 'axios';
import { v4 as uuid } from 'uuid';
import glob from 'glob';
import potrace from 'potrace';
import { flattenSVG } from 'flatten-svg';
import { createSVGWindow } from 'svgdom';
import sharp from 'sharp';
import TextToSVG from 'text-to-svg';
import * as poly2tri from 'poly2tri';
import { countBy, flatten, flattenDeep, groupBy, max, maxBy, min, orderBy, uniqBy } from 'lodash';
import * as SpriteSmith from 'spritesmith';
import {
  Colour,
  colourToHex,
  flattenPolygon,
  randomColour,
  Rectangle,
  Sprite,
  SpriteSheetEntry,
  SpriteType,
} from './types';
import simplify from 'simplify-js';
import { SVG } from '@svgdotjs/svg.js';
import skmeans from 'skmeans';
import { config } from '../package.json';

// eslint-disable-next-line @typescript-eslint/no-var-requires
const { registerWindow } = require('@svgdotjs/svg.js');

const runSpriteSmith = util.promisify(SpriteSmith.run);
const exists = util.promisify(fs.exists);
const mkdir = util.promisify(fs.mkdir);
const rmdir = util.promisify(fs.rmdir);
const writeFile = util.promisify(fs.writeFile);
const copyFile = util.promisify(fs.copyFile);
const globP = util.promisify(glob);

const fontPath = path.join(__dirname, '..', 'fonts', 'font.ttf');
const distPath = path.join(__dirname, '..', 'dist');
const spritesPath = path.join(distPath, 'sprites');
const svgPath = path.join(distPath, 'svg');
let textToSvgCache: TextToSVG.TextToSVG | null = null;

async function getTextToSvg(): Promise<TextToSVG.TextToSVG> {
  if (textToSvgCache) {
    return textToSvgCache;
  }

  if (!(await exists(fontPath))) {
    console.log(`downloading ${config.fontUrl} -> ${fontPath}`);
    const response = await axios.get(config.fontUrl, {
      responseType: 'arraybuffer',
    });
    await writeFile(fontPath, Buffer.from(response.data));
  }

  return (textToSvgCache = TextToSVG.loadSync(fontPath));
}

function getSpritePath(id: string): string {
  return path.join(spritesPath, `${id}.png`);
}

async function getImagePaths(): Promise<string[]> {
  const root = path.join(__dirname, '..', 'images', '**', '*.png');
  return await globP(root, {});
}

function isHole(aabb: Rectangle, hole: Rectangle) {
  function between(a: number, lower: number, upper: number) {
    return a >= lower && a <= upper;
  }

  return (
    between(hole.x, aabb.x, aabb.p1.x) &&
    between(hole.y, aabb.y, aabb.p1.y) &&
    between(hole.p1.x, aabb.x, aabb.p1.x) &&
    between(hole.p1.y, aabb.y, aabb.p1.y)
  );
}

function getAABB(points: number[][]): Rectangle {
  const xs = points.map(x => x[0]);
  const ys = points.map(x => x[1]);
  const x = min(xs) || 0;
  const y = min(ys) || 0;
  const x2 = max(xs) || 0;
  const y2 = max(ys) || 0;

  return {
    x,
    y,
    p1: { x: x2, y: y2 },
    width: x2 - x,
    height: y2 - y,
  };
}

async function triangulateSvg(type: SpriteType, name: string, svg: string, colour: Colour): Promise<Sprite> {
  const window = createSVGWindow();
  const document = window.document.documentElement;
  document.innerHTML = svg;
  const flatPaths = flattenSVG(document, { maxError: 1 });

  const aabb = getAABB(flatten(flatPaths.map(path => path.points)));

  const scaledPaths = flatPaths.map(path => {
    const scale = 1 / Math.max(aabb.width, aabb.height);
    const points = uniqBy(
      path.points.map(([x, y]) => [(x - aabb.x) * scale, 1 - (y - aabb.y) * scale]),
      ([x, y]) => `${x}_${y}`,
    );
    return {
      points,
      aabb: getAABB(points),
      holes: [] as number[][][],
    };
  });

  for (const path of scaledPaths) {
    const holes = scaledPaths.filter(p => p !== path && isHole(path.aabb, p.aabb));
    path.holes = holes.map(h => h.points);
    for (const hole of holes) {
      const index = scaledPaths.indexOf(hole);
      scaledPaths.splice(index, 1);
    }
  }

  function toPoints(pts: number[][]) {
    return simplify(
      pts.map(p => new poly2tri.Point(p[0], p[1])),
      0.01,
      true,
    );
  }

  const polygons = scaledPaths
    .map(({ points, holes }) => {
      const contour = toPoints(points);
      const sweep = new poly2tri.SweepContext(contour);
      /*
      for (const hole of holes) {
        const holeContour = toPoints(hole);
        sweep.addHole(holeContour);
      }*/
      try {
        sweep.triangulate();
      } catch (e) {
        console.error(`failed to triangulate ${name}`, e);
        throw e;
      }
      return sweep.getTriangles().map(tri => tri.getPoints().reverse());
    })
    .reduce((x, y) => [...x, ...y], []);

  document.innerHTML = '';
  registerWindow(window, window.document);
  const canvas: any = SVG(document.documentElement);
  const scale = 100;
  const flatPoints = flattenDeep(polygons);
  canvas.viewbox(0, 0, scale * (max(flatPoints.map(x => x.x)) || 1), scale * (max(flatPoints.map(x => x.y)) || 1));

  for (const polygon of polygons) {
    const s = polygon.map(({ x, y }) => `${x * scale},${y * scale}`).join(' ');
    canvas.polygon(s).fill('white').stroke({ color: 'black', width: 1 });
  }

  await writeFile(path.join(svgPath, `${name}.svg`), canvas.svg(), { encoding: 'utf8' });

  return {
    id: uuid(),
    type,
    name,
    svg,
    colour,
    polygons,
    size: { width: aabb.width, height: aabb.height },
  };
}

async function getDominantColour(imagePath: string): Promise<Colour> {
  const { data, info } = await sharp(imagePath).raw().toBuffer({ resolveWithObject: true });

  const rgbs: number[][] = [];
  for (let i = 0; i < info.size; i += info.channels) {
    const rgb = data.slice(i, i + 3);
    if (rgb.every(i => i === 0) || rgb.every(i => i === 255)) {
      continue;
    }
    rgbs.push([...rgb]);
  }

  const kmeans = skmeans(rgbs, 3);
  const { id: clusterId } = maxBy(
    Object.entries(countBy(kmeans.idxs)).map(([id, count]) => ({ id: parseInt(id, 10), count })),
    x => x.count,
  ) || { id: 0 };
  const [r, g, b] = (kmeans.centroids[clusterId] as number[]).map(x => Math.round(x));
  return { r, g, b };
}

async function triangulateImage(imagePath: string): Promise<Sprite> {
  // TODO this dominant colour is the transparent colour sometimes...
  const stats = await sharp(imagePath).stats();

  const params = {
    color: 'black',
    threshold: 254, // quite high as we're only interested in the model outline.
  };
  return new Promise((resolve, reject) =>
    potrace.trace(imagePath, params, (err, svg) => {
      if (err) {
        reject(err);
        return;
      }
      const name = path.basename(imagePath, path.extname(imagePath));
      resolve(triangulateSvg(SpriteType.Image, name, svg, stats.dominant));
    }),
  );
}

async function triangulateChar(char: string, textToSvg: TextToSVG.TextToSVG): Promise<Sprite> {
  if (char.length !== 1) {
    throw new Error('must provide single char');
  }

  const colour = randomColour();
  const fill = colourToHex(colour);
  const attributes = { fill, stroke: fill };
  const options = { x: 0, y: 0, fontSize: 500, anchor: 'top', attributes: attributes };

  const svg = textToSvg.getSVG(char, options);
  return await triangulateSvg(SpriteType.Character, char, svg, colour);
}

async function rasterizeSvg(sprite: Sprite) {
  await sharp(Buffer.from(sprite.svg))
    .trim()
    .png()
    .toFile(path.join(spritesPath, `${sprite.id}.png`));
}

async function main() {
  if (await exists(distPath)) {
    await rmdir(distPath, { recursive: true });
  }
  await mkdir(distPath);
  await mkdir(spritesPath);
  await mkdir(svgPath);

  const paths = await getImagePaths();
  const imageSprites = await Promise.all(
    paths.map(async p => {
      try {
        const image = await triangulateImage(p);
        await copyFile(p, getSpritePath(image.id));
        return image;
      } catch (err) {
        console.log(`failed to triangulate ${p}`);
        throw err;
      }
    }),
  );

  const textToSvg = await getTextToSvg();
  const ALL = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890`¬!"£$%^&*()-_=+[]{};:\'@#~,<.>/?';
  const charSprites = await Promise.all([...ALL].map(async ch => await triangulateChar(ch, textToSvg)));

  for (const sprite of charSprites) {
    await rasterizeSvg(sprite);
  }

  const allSprites = [...charSprites, ...imageSprites];

  const spriteSheet = await runSpriteSmith({
    src: allSprites.map(sprite => getSpritePath(sprite.id)),
  });

  const entries: (SpriteSheetEntry & { name?: string })[] = allSprites.map(sprite => {
    const { x, y, width, height } = spriteSheet.coordinates[getSpritePath(sprite.id)];
    return {
      name: sprite.name,
      polygons: sprite.polygons.map(flattenPolygon),
      position: { x, y },
      size: { width, height },
      colour: sprite.colour,
    };
  });
  const result = groupBy(entries, x => x.name?.toLowerCase()[0]);

  const biggestSprites = orderBy(allSprites, x => x.polygons.length, 'desc')
    .slice(0, 5)
    .map(x => ({ name: x.name, length: x.polygons.length }));
  console.log('biggest sprites', biggestSprites);

  await rmdir(spritesPath, { recursive: true });
  await writeFile(path.join(distPath, 'sprites.png'), spriteSheet.image, 'binary');
  await writeFile(path.join(distPath, 'sprites.json'), JSON.stringify(result), { encoding: 'utf8' });
}

main()
  .then(() => process.exit(0))
  .catch(err => {
    console.error(err);
    process.exit(1);
  });
