type ImageMeta = {
  images?: Omit<ImageMeta, "images">[];
  width: number;
  height: number;
  orientation?: number;
  type?: string;
};
/**
 * @param {Uint8Array|string} input - Uint8Array or relative/absolute path of the image file
 * @param {Function=} [callback] - optional function for async detection
 */
export declare function imageMeta(input: Uint8Array): ImageMeta;
export type { ImageMeta };