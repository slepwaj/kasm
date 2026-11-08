export const create: (dictPath: string) => number;
export const key: (handle: number, ch: number) => string;
export const backspace: (handle: number) => string;
export const space: (handle: number) => string;
export const destroy: (handle: number) => void;
export const select: (handle: number, index: number) => string;
export const setScheme: (handle: number, name: string) => boolean;
