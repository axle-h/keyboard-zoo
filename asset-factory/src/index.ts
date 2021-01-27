import * as path from 'path';
import * as fs from 'fs';
import * as util from 'util';
import { v4 as uuid } from 'uuid';
import glob from 'glob';
import potrace from 'potrace';
import { flattenSVG } from 'flatten-svg';
import { createSVGWindow } from 'svgdom';
import sharp from 'sharp';
import TextToSVG from 'text-to-svg';
import * as poly2tri from 'poly2tri';
import { min, max, flatten, flattenDeep, uniqBy, groupBy } from 'lodash';
import * as SpriteSmith from 'spritesmith';
import { Colour, colourToHex, randomColour, Rectangle, Sprite, SpriteSheetEntry, SpriteType } from './types';

const runSpriteSmith = util.promisify(SpriteSmith.run);
const exists = util.promisify(fs.exists);
const mkdir = util.promisify(fs.mkdir);
const rmdir = util.promisify(fs.rmdir);
const writeFile = util.promisify(fs.writeFile);
const copyFile = util.promisify(fs.copyFile);
const textToSVG = TextToSVG.loadSync(path.join(__dirname, '..', 'fonts', 'Roboto', 'Roboto-Regular.ttf'));
const globP = util.promisify(glob);

const distPath = path.join(__dirname, '..', 'dist');
const spritesPath = path.join(distPath, 'sprites');
const svgPath = path.join(distPath, 'svg');

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

function triangulateSvg(type: SpriteType, name: string, svg: string, colour: Colour): Sprite {
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

  const polygons = scaledPaths
    .map(({ points, holes }) => {
      const contour = points.map(p => new poly2tri.Point(p[0], p[1]));
      const sweep = new poly2tri.SweepContext(contour);
      for (const hole of holes) {
        const holeContour = hole.map(p => new poly2tri.Point(p[0], p[1]));
        sweep.addHole(holeContour);
      }
      sweep.triangulate();
      return sweep.getTriangles().map(tri =>
        tri
          .getPoints()
          .map(p => [p.x, p.y])
          .reverse()
          .reduce((x, y) => [...x, ...y], []),
      );
    })
    .reduce((x, y) => [...x, ...y], []);

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

async function triangulateImage(imagePath: string): Promise<Sprite> {
  // TODO this dominant colour is the transparent colour sometimes...
  const { dominant } = await sharp(imagePath).stats();
  const params = {
    color: 'black',
    threshold: 140, // quite high as we're only interested in the model outline.
  };
  return new Promise((resolve, reject) =>
    potrace.trace(imagePath, params, (err, svg) => {
      if (err) {
        reject(err);
        return;
      }
      const name = path.basename(imagePath, path.extname(imagePath));
      resolve(triangulateSvg(SpriteType.Image, name, svg, dominant));
    }),
  );
}

async function triangulateChar(char: string): Promise<Sprite> {
  if (char.length !== 1) {
    throw new Error('must provide single char');
  }

  const colour = randomColour();
  const fill = colourToHex(colour);
  const attributes = { fill, stroke: fill };
  const options = { x: 0, y: 0, fontSize: 500, anchor: 'top', attributes: attributes };
  const svg = textToSVG.getSVG(char, options);
  return triangulateSvg(SpriteType.Character, char, svg, colour);
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
      const image = await triangulateImage(p);
      await copyFile(p, getSpritePath(image.id));
      return image;
    }),
  );

  const ALL = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890`¬!"£$%^&*()-_=+[]{};:\'@#~,<.>/?';
  const sprites = await Promise.all([...ALL].map(ch => triangulateChar(ch)));

  for (const sprite of sprites) {
    await rasterizeSvg(sprite);
  }

  const allSprites = [...sprites, ...imageSprites];

  const spriteSheet = await runSpriteSmith({
    src: allSprites.map(sprite => getSpritePath(sprite.id)),
  });

  const entries: (SpriteSheetEntry & { name?: string })[] = allSprites.map(sprite => {
    const { x, y, width, height } = spriteSheet.coordinates[getSpritePath(sprite.id)];
    return {
      name: sprite.name,
      polygons: sprite.polygons,
      position: { x, y },
      size: { width, height },
      colour: sprite.colour,
    };
  });
  const result = groupBy(entries, x => x.name?.toLowerCase()[0]);

  for (const grp of Object.values(result)) {
    for (const entry of grp) {
      delete entry.name;
    }
  }

  for (const sprite of allSprites.filter(x => x.type === SpriteType.Image)) {
    await writeFile(path.join(svgPath, `${sprite.name}.svg`), sprite.svg, { encoding: 'utf8' });
  }

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
